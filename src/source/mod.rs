pub mod async_receiver;
pub mod combiner;
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

use crate::{Error, Loop, RangeSource};
use std::sync::atomic::{AtomicU64, Ordering};

const START_GENERATED_NODE_IDS: u64 = 0x10000;
static NEXT_ID: AtomicU64 = AtomicU64::new(START_GENERATED_NODE_IDS);

pub trait Node {
    fn get_node_id(&self) -> u64;
    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error>;
    fn on_event(&mut self, event: &NodeEvent);
    fn fill_buffer(&mut self, buffer: &mut [f32]);
    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error>;

    fn new_node_id() -> u64
    where
        Self: Sized,
    {
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }
}

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
