use crate::{
    AssetLoader, Error, Event, EventTarget, GraphNode, Message,
    abstraction::NodeRegistry,
    config::{ChildConfig, builtin::register_builtin_types, registry::init_node_registry},
    consts,
    event::EventTiming,
    generator::NullNode,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossbeam_channel::{Receiver, SendError, Sender, bounded, unbounded};
use serde_json::Value;
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

type SnapshotResponse = Option<Result<Value, Error>>;

struct AudioClock {
    current_rendering_absolute_frame: AtomicU64,
}

impl AudioClock {
    fn new() -> Self {
        Self {
            current_rendering_absolute_frame: AtomicU64::new(0),
        }
    }

    fn reset(&self) {
        self.current_rendering_absolute_frame
            .store(0, Ordering::Relaxed);
    }

    fn get_current_absolute_frame(&self) -> u64 {
        self.current_rendering_absolute_frame
            .load(Ordering::Relaxed)
    }

    fn set_current_absolute_frame(&self, frame: u64) {
        self.current_rendering_absolute_frame
            .store(frame, Ordering::Relaxed);
    }
}

enum AudioCommand {
    GraphMessage(Message),
    SwapConsumer {
        consumer: GraphNode,
        response_sender: Sender<GraphNode>,
    },
    GetStateSnapshot {
        node_id: u64,
        response_sender: Sender<SnapshotResponse>,
    },
}

enum SwapConsumerError {
    Send { error: Error, consumer: GraphNode },
    Receive(Error),
}

#[derive(Clone)]
pub struct MessageSender {
    command_sender: Sender<AudioCommand>,
    clock: Arc<AudioClock>,
}

impl MessageSender {
    fn new(command_sender: Sender<AudioCommand>, clock: Arc<AudioClock>) -> Self {
        Self {
            command_sender,
            clock,
        }
    }

    pub fn current_rendering_absolute_frame(&self) -> u64 {
        self.clock
            .current_rendering_absolute_frame
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn send(&self, message: Message) -> Result<(), SendError<Message>> {
        match self
            .command_sender
            .send(AudioCommand::GraphMessage(message))
        {
            Ok(()) => Ok(()),
            Err(SendError(AudioCommand::GraphMessage(message))) => Err(SendError(message)),
            Err(SendError(_)) => unreachable!("MessageSender only sends event commands"),
        }
    }
}

enum ConsumerCell {
    Source(GraphNode),
    Placeholder,
}

struct ScheduledMessageEvent {
    target: EventTarget,
    data: Event,
    absolute_frame: u64,
}

impl PartialEq for ScheduledMessageEvent {
    fn eq(&self, other: &Self) -> bool {
        self.absolute_frame == other.absolute_frame
    }
}

impl Eq for ScheduledMessageEvent {}

impl PartialOrd for ScheduledMessageEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.absolute_frame
            .partial_cmp(&other.absolute_frame)
            .map(|ord| ord.reverse())
    }
}

impl Ord for ScheduledMessageEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.absolute_frame.cmp(&other.absolute_frame).reverse()
    }
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
    command_sender: Sender<AudioCommand>,
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
        let clock = Arc::new(AudioClock::new());
        let program_sources = programs
            .into_iter()
            .map(|(program, node)| (program, ConsumerCell::Source(node)))
            .collect::<HashMap<usize, ConsumerCell>>();
        let (command_sender, command_receiver) = unbounded();
        let stream = Self::open_stream(null_node, command_receiver, clock.clone())?;
        stream.play()?;
        let mut mixer = Self {
            stream: Mutex::new(stream),
            program_sources,
            event_sender: Arc::new(MessageSender::new(command_sender.clone(), clock)),
            command_sender,
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
            match self.swap_active_consumer(program) {
                Ok(previous_program) => {
                    drop(previous_program);
                    return true;
                }
                Err(SwapConsumerError::Send { error, consumer }) => {
                    drop(consumer);
                    println!("ERROR: Mixer: Could not replace active program: {}", error);
                    return false;
                }
                Err(SwapConsumerError::Receive(error)) => {
                    println!("ERROR: Mixer: Could not replace active program: {}", error);
                    return false;
                }
            }
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

        let previous_program = match self.swap_active_consumer(new_program) {
            Ok(previous_program) => previous_program,
            Err(SwapConsumerError::Send { error, consumer }) => {
                self.program_sources
                    .insert(program_no, ConsumerCell::Source(consumer));
                return Err(error);
            }
            Err(SwapConsumerError::Receive(error)) => {
                return Err(error);
            }
        };

        self.program_sources
            .insert(program_no, ConsumerCell::Placeholder);
        if let Some(index) = existing_placeholder_index {
            self.program_sources
                .insert(index, ConsumerCell::Source(previous_program));
        }

        Ok(())
    }

    pub fn get_current_program_no(&self) -> Option<usize> {
        self.program_sources.iter().find_map(|(k, v)| match v {
            &ConsumerCell::Placeholder => Some(*k),
            _ => None,
        })
    }

    pub fn get_active_node_state_snapshot(&self, node_id: u64) -> Option<Result<Value, Error>> {
        let (response_sender, response_receiver) = bounded(1);
        if self
            .command_sender
            .send(AudioCommand::GetStateSnapshot {
                node_id,
                response_sender,
            })
            .is_err()
        {
            return Some(Err(Error::Internal(
                "Could not request node state snapshot: audio thread is unavailable".to_owned(),
            )));
        }
        response_receiver.recv().unwrap_or_else(|_| {
            Some(Err(Error::Internal(
                "Could not receive node state snapshot: audio thread is unavailable".to_owned(),
            )))
        })
    }

    fn swap_active_consumer(&self, consumer: GraphNode) -> Result<GraphNode, SwapConsumerError> {
        let (response_sender, response_receiver) = bounded(1);
        let command = AudioCommand::SwapConsumer {
            consumer,
            response_sender,
        };
        if let Err(SendError(command)) = self.command_sender.send(command) {
            return match command {
                AudioCommand::SwapConsumer { consumer, .. } => Err(SwapConsumerError::Send {
                    error: Error::Internal(
                        "Could not change program: audio thread is unavailable".to_owned(),
                    ),
                    consumer,
                }),
                _ => unreachable!("swap_active_consumer only sends swap commands"),
            };
        }
        response_receiver.recv().map_err(|_| {
            SwapConsumerError::Receive(Error::Internal(
                "Could not change program: audio thread did not return the active program"
                    .to_owned(),
            ))
        })
    }

    fn open_stream(
        mut consumer: GraphNode,
        command_receiver: Receiver<AudioCommand>,
        clock: Arc<AudioClock>,
    ) -> Result<Stream, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(consts::BUFFER_SIZE as u32),
            channels: consts::CHANNEL_COUNT as u16,
            sample_rate: cpal::SampleRate(consts::PLAYBACK_SAMPLE_RATE as u32),
        };

        let mut pending_messages: BinaryHeap<ScheduledMessageEvent> = BinaryHeap::new();

        clock.reset();
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                data.fill(0.0);

                let buffer_frames = data.len() / consts::CHANNEL_COUNT;
                let buffer_start_frame = clock.get_current_absolute_frame();
                let buffer_end_frame = buffer_start_frame + buffer_frames as u64;

                for command in command_receiver.try_iter() {
                    match command {
                        AudioCommand::GraphMessage(message) => match message.timing {
                            EventTiming::Imprecise => {
                                consumer.on_event(&message);
                            }
                            EventTiming::AtAbsoluteFrame(absolute_frame) => {
                                pending_messages.push(ScheduledMessageEvent {
                                    target: message.target,
                                    data: message.data,
                                    absolute_frame: absolute_frame,
                                });
                            }
                        },
                        AudioCommand::SwapConsumer {
                            consumer: new_consumer,
                            response_sender,
                        } => {
                            let previous_consumer = std::mem::replace(&mut consumer, new_consumer);
                            let _ = response_sender.try_send(previous_consumer);
                        }
                        AudioCommand::GetStateSnapshot {
                            node_id,
                            response_sender,
                        } => {
                            let _ = response_sender.try_send(consumer.get_state_snapshot(node_id));
                        }
                    }
                }

                let mut cursor_offset_frame: usize = 0;
                while let Some(next_message) = pending_messages.peek() {
                    let message_frame = next_message.absolute_frame.max(buffer_start_frame);
                    if message_frame >= buffer_end_frame {
                        break;
                    }
                    if next_message.absolute_frame < buffer_start_frame {
                        println!(
                            "WARNING: Message processed late ({} < {})",
                            next_message.absolute_frame, buffer_start_frame
                        );
                    }
                    let buffer_offset_frame = (message_frame - buffer_start_frame) as usize;
                    let samples_start = cursor_offset_frame * consts::CHANNEL_COUNT;
                    let samples_end = buffer_offset_frame * consts::CHANNEL_COUNT;
                    consumer.fill_buffer(&mut data[samples_start..samples_end]);
                    cursor_offset_frame = buffer_offset_frame;

                    while pending_messages
                        .peek()
                        .is_some_and(|message| message.absolute_frame <= message_frame)
                    {
                        let message = pending_messages.pop().unwrap();
                        consumer.on_event(&Message {
                            target: message.target,
                            data: message.data,
                            timing: EventTiming::AtAbsoluteFrame(
                                buffer_start_frame + cursor_offset_frame as u64,
                            ),
                        });
                    }
                }

                consumer.fill_buffer(&mut data[(cursor_offset_frame * consts::CHANNEL_COUNT)..]);
                clock.set_current_absolute_frame(buffer_end_frame);
            },
            move |err| {
                println!("ERROR: Stream: {:?}", err);
            },
            None,
        )?;
        Ok(stream)
    }
}
