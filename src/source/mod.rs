pub mod async_receiver;
pub mod envelope;
pub mod fader;
pub mod font;
pub mod midi;
pub mod mixer;
pub mod noise;
pub mod null;
pub mod one_shot;
pub mod sawtooth;
pub mod square;
pub mod triangle;
pub mod util;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

use crate::{
    util::{one_shot_from_file, wav_from_file},
    Error, Loop, RangeSource, SoundFont, SoundSource,
};
use std::sync::atomic::{AtomicU64, Ordering};

const START_GENERATED_NODE_IDS: u64 = 0x10000;
static NEXT_ID: AtomicU64 = AtomicU64::new(START_GENERATED_NODE_IDS);

pub trait Node {
    fn get_node_id(&self) -> u64;
    fn on_event(&mut self, event: &NodeEvent);
    fn fill_buffer(&mut self, buffer: &mut [f32]);

    fn new_node_id() -> u64
    where
        Self: Sized,
    {
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }
}

pub trait BufferConsumer {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error>;
}

pub trait BufferConsumerNode: BufferConsumer + Node {}

#[derive(Clone)]
pub struct NoteRange {
    pub lower_inclusive: u8,
    pub upper_inclusive: u8,
}

impl NoteRange {
    pub fn new_inclusive_range(lower: u8, upper: u8) -> Self {
        Self {
            lower_inclusive: lower,
            upper_inclusive: upper,
        }
    }

    pub fn new_full_range() -> Self {
        Self {
            lower_inclusive: 0,
            upper_inclusive: 255,
        }
    }

    pub fn from_config(config: &RangeSource) -> Self {
        Self {
            lower_inclusive: config.lower,
            upper_inclusive: config.upper,
        }
    }

    pub fn contains(&self, note: u8) -> bool {
        self.lower_inclusive <= note && self.upper_inclusive >= note
    }
}

#[derive(Clone, Debug)]
pub enum NodeEvent {
    Broadcast(BroadcastControl),
    Note {
        note: u8,
        event: NoteEvent,
    },
    NodeControl {
        node_id: u64,
        event: NodeControlEvent,
    },
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum BroadcastControl {
    NotesOff,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum NoteEvent {
    NoteOn { vel: f32 },
    NoteOff { vel: f32 },
}

#[derive(Clone, Debug)]
pub enum NodeControlEvent {
    MixerBalance(f32),
    Volume(f32),
    Fade { from: f32, to: f32, seconds: f32 },
    SeekWhenIdeal { to_anchor: Option<u32> },
    Unknown,
}

pub struct LoopRange {
    pub start_frame: usize,
    pub end_frame: usize,
}

impl LoopRange {
    pub fn new_frame_range(start_frame: usize, end_frame: usize) -> Self {
        Self {
            start_frame,
            end_frame,
        }
    }

    pub fn from_config(config: &Loop) -> Self {
        Self {
            start_frame: config.start,
            end_frame: config.end,
        }
    }
}

pub fn source_from_config(
    config: &SoundSource,
) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
    let consumer: Box<dyn BufferConsumerNode + Send + 'static> = match config {
        SoundSource::Midi {
            node_id,
            source,
            channels,
        } => Box::new(midi::MidiSource::from_config(*node_id, source, channels)?),
        SoundSource::Font { node_id, config } => {
            Box::new(SoundFont::from_config(*node_id, config)?)
        }
        SoundSource::SquareWave {
            node_id,
            amplitude,
            duty_cycle,
        } => Box::new(square::SquareWaveSource::new(
            *node_id,
            *amplitude,
            *duty_cycle,
        )),
        SoundSource::TriangleWave { node_id, amplitude } => {
            Box::new(triangle::TriangleWaveSource::new(*node_id, *amplitude))
        }
        SoundSource::SawtoothWave { node_id, amplitude } => {
            Box::new(sawtooth::SawtoothWaveSource::new(*node_id, *amplitude))
        }
        SoundSource::LfsrNoise {
            node_id,
            amplitude,
            inside_feedback,
            note_for_16_shifts,
        } => Box::new(noise::LfsrNoiseSource::new(
            *node_id,
            *amplitude,
            *inside_feedback,
            *note_for_16_shifts,
        )),
        SoundSource::SampleFilePath {
            node_id,
            path,
            base_note,
            looping,
        } => {
            let loop_range = looping.as_ref().map(LoopRange::from_config);
            Box::new(wav_from_file(
                path.as_str(),
                *base_note,
                loop_range,
                *node_id,
            )?)
        }
        SoundSource::OneShotFilePath { node_id, path } => {
            Box::new(one_shot_from_file(path.as_str(), *node_id)?)
        }
        SoundSource::Envelope {
            node_id,
            attack_time,
            decay_time,
            sustain_multiplier,
            release_time,
            source,
        } => {
            let consumer = source_from_config(source)?;
            Box::new(envelope::Envelope::from_adsr(
                *node_id,
                *attack_time,
                *decay_time,
                *sustain_multiplier,
                *release_time,
                consumer,
            ))
        }
        SoundSource::Mixer {
            node_id,
            balance,
            source_0,
            source_1,
        } => {
            let consumer_0 = source_from_config(source_0)?;
            let consumer_1 = source_from_config(source_1)?;
            Box::new(mixer::MixerSource::new(
                *node_id, *balance, consumer_0, consumer_1,
            ))
        }
        SoundSource::Fader {
            node_id,
            initial_volume,
            source,
        } => {
            let consumer = source_from_config(source)?;
            Box::new(fader::Fader::new(*node_id, *initial_volume, consumer))
        }
    };
    Ok(consumer)
}
