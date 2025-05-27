use crate::{
    Config, Error, GraphLoader, GraphNode, Message, MessageSender, consts, generator::NullSource,
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
    pub fn start_empty() -> Result<Self, Error> {
        let consumer = Box::new(NullSource::new(None));
        Self::start_single_program(consumer)
    }

    pub fn start_single_program(consumer: GraphNode) -> Result<Self, Error> {
        let swappable = super::swap::SwappableConsumer::new(consumer);
        let (event_sender, event_receiver) = unbounded();
        let stream = Self::open_stream(swappable.take_consumer(), event_receiver)?;
        stream.play()?;
        Ok(Self {
            stream: Mutex::new(stream),
            program_sources: HashMap::new(),
            event_sender: Arc::new(event_sender),
            consumer: swappable,
        })
    }

    pub fn start_single_program_from_config<L: GraphLoader>(
        loader: &L,
        program_no: Option<usize>,
        config: &Config,
    ) -> Result<Self, Error> {
        let source = loader.load_source_with_dependencies(&config.root)?;
        if let Some(program_no) = &program_no {
            let mut mixer = Self::start_empty()?;
            mixer.store_program(*program_no, source);
            mixer.change_program(*program_no)?;
            Ok(mixer)
        } else {
            let mixer = Self::start_single_program(source)?;
            Ok(mixer)
        }
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
