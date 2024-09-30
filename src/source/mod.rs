pub mod async_receiver;
pub mod envelope;
pub mod font;
pub mod midi;
pub mod noise;
pub mod sawtooth;
pub mod square;
pub mod triangle;
pub mod util;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

use crate::{Error, Loop, RangeSource};

pub trait BufferConsumer {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error>;
    fn set_note(&mut self, event: NoteEvent);
    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status;
}

pub trait NoteConsumer {
    fn restart_with_event(&mut self, event: &NoteEvent);
    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status;
}

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

#[derive(PartialEq)]
pub enum Status {
    Ok,
    Ended,
}

pub struct NoteEvent {
    pub kind: NoteKind,
}

#[derive(PartialEq, Copy, Clone)]
pub enum NoteKind {
    NoteOn { note: u8, vel: f32 },
    NoteOff { note: u8, vel: f32 },
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
