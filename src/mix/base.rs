use crate::{
    AssetLoader, Error, GraphNode, Message, MessageSender,
    abstraction::NodeRegistry,
    config::{ChildConfig, builtin::register_builtin_types, registry::init_node_registry},
    consts,
    generator::NullNode,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossbeam_channel::{Receiver, unbounded};
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicPtr, Ordering},
};

enum ConsumerCell {
    Source(GraphNode),
    Placeholder,
}

pub struct BaseMixerBuilder {
    programs: HashMap<usize, GraphNode>,
    initial_program: Option<usize>,
}

impl BaseMixerBuilder {
    pub(crate) fn with_new_registry<F>(mut customisation: F) -> Result<Self, Error>
    where
        F: FnMut(&mut NodeRegistry),
    {
        let mut config_registry = NodeRegistry::new();
        register_builtin_types(&mut config_registry);
        customisation(&mut config_registry);
        init_node_registry(config_registry)?;
        Ok(Self::with_existing_registry())
    }

    pub(crate) fn with_existing_registry() -> Self {
        Self {
            programs: HashMap::new(),
            initial_program: None,
        }
    }

    pub fn store_program(mut self, program_no: usize, node: GraphNode) -> Self {
        self.programs.insert(program_no, node);
        self
    }

    pub fn set_initial_program(mut self, program_no: usize, node: GraphNode) -> Self {
        self.initial_program = Some(program_no);
        self.store_program(program_no, node)
    }

    pub fn store_program_from_config(
        mut self,
        program_no: usize,
        config: ChildConfig,
        asset_loader: &mut dyn AssetLoader,
    ) -> Result<Self, Error> {
        let node = config.0.to_node(asset_loader)?;
        self.programs.insert(program_no, node);
        Ok(self)
    }

    pub fn set_initial_program_from_config(
        mut self,
        program_no: usize,
        config: ChildConfig,
        asset_loader: &mut dyn AssetLoader,
    ) -> Result<Self, Error> {
        self.initial_program = Some(program_no);
        self.store_program_from_config(program_no, config, asset_loader)
    }

    pub fn start(self, initial_program_no: Option<usize>) -> Result<BaseMixer, Error> {
        BaseMixer::start_new(self.programs, initial_program_no)
    }
}

pub struct BaseMixer {
    stream: Mutex<Stream>,
    program_sources: HashMap<usize, ConsumerCell>,
    event_sender: Arc<MessageSender>,
    consumer: super::swap::SwappableConsumer,
}

impl Drop for BaseMixer {
    fn drop(&mut self) {
        let stream = self.stream.lock().expect("Could not lock the audio stream");
        stream.pause().expect("Could not pause the stream");
    }
}

impl BaseMixer {
    pub fn builder_with_default_registry() -> Result<BaseMixerBuilder, Error> {
        BaseMixerBuilder::with_new_registry(|_| {})
    }

    pub fn builder_with_custom_registry<F>(customisation: F) -> Result<BaseMixerBuilder, Error>
    where
        F: FnMut(&mut NodeRegistry),
    {
        BaseMixerBuilder::with_new_registry(customisation)
    }

    pub fn builder_with_existing_registry() -> BaseMixerBuilder {
        BaseMixerBuilder::with_existing_registry()
    }

    pub(crate) fn start_new(
        programs: HashMap<usize, GraphNode>,
        initial_program_no: Option<usize>,
    ) -> Result<Self, Error> {
        let null_node = Box::new(NullNode::new(None));
        let swappable = super::swap::SwappableConsumer::new(null_node);
        let program_sources = programs
            .into_iter()
            .map(|(program, node)| (program, ConsumerCell::Source(node)))
            .collect::<HashMap<usize, ConsumerCell>>();
        let (event_sender, event_receiver) = unbounded();
        let stream = Self::open_stream(swappable.take_consumer(), event_receiver)?;
        stream.play()?;
        let mut mixer = Self {
            stream: Mutex::new(stream),
            program_sources,
            event_sender: Arc::new(event_sender),
            consumer: swappable,
        };
        if let Some(program) = initial_program_no {
            mixer.change_program(program)?;
        }
        Ok(mixer)
    }

    pub fn get_event_sender(&self) -> Arc<MessageSender> {
        self.event_sender.clone()
    }

    // Store a program at a given index.
    // Return whether a program already existed in that index (and will be replaced).
    pub fn store_program(&mut self, program_no: usize, program: GraphNode) -> bool {
        // A program is already at this index and is currently being played; it will be discarded
        if matches!(
            self.program_sources.get(&program_no),
            Some(&ConsumerCell::Placeholder)
        ) {
            self.consumer.swap_consumer(program);
            return true;
        }

        // Either no program yet at this index, or it's not currently playing and will be discarded
        let cell = ConsumerCell::Source(program);
        let previous = self.program_sources.insert(program_no, cell);
        previous.is_some()
    }

    pub fn change_program(&mut self, program_no: usize) -> Result<(), Error> {
        let existing_placeholder_index = self.get_current_program_no();

        let new_program = match self.program_sources.remove(&program_no) {
            Some(ConsumerCell::Placeholder) => {
                return Err(Error::User(format!(
                    "Cannot change program: program no. {} is already playing",
                    program_no
                )));
            }
            Some(ConsumerCell::Source(program)) => program,
            None => {
                return Err(Error::User(format!(
                    "Cannot change program: nothing is stored for program no. {}",
                    program_no
                )));
            }
        };

        self.program_sources
            .insert(program_no, ConsumerCell::Placeholder);
        if let Some(previous_program) = self.consumer.swap_consumer(new_program) {
            if let Some(index) = existing_placeholder_index {
                self.program_sources
                    .insert(index, ConsumerCell::Source(previous_program));
            }
        }

        Ok(())
    }

    pub fn get_current_program_no(&self) -> Option<usize> {
        self.program_sources.iter().find_map(|(k, v)| match v {
            &ConsumerCell::Placeholder => Some(*k),
            _ => None,
        })
    }

    fn open_stream(
        consumer: Arc<AtomicPtr<GraphNode>>,
        event_receiver: Receiver<Message>,
    ) -> Result<Stream, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(consts::BUFFER_SIZE as u32),
            channels: consts::CHANNEL_COUNT as u16,
            sample_rate: cpal::SampleRate(consts::PLAYBACK_SAMPLE_RATE as u32),
        };
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                data.fill(0.0);
                let consumer_ptr = consumer.load(Ordering::SeqCst);
                if !consumer_ptr.is_null() {
                    unsafe {
                        for event in event_receiver.try_iter() {
                            (*consumer_ptr).on_event(&event);
                        }
                        (*consumer_ptr).fill_buffer(data);
                    }
                }
            },
            move |err| {
                println!("ERROR: Stream: {:?}", err);
            },
            None,
        )?;
        Ok(stream)
    }
}
