pub mod font;
pub mod midi;
pub mod noise;
pub mod square;
pub mod triangle;
pub mod util;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

use crate::Error;

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
    lower_inclusive: u8,
    upper_inclusive: u8,
}

impl NoteRange {
    pub fn new_inclusive_range(lower: u8, upper: u8) -> Self {
        Self {
            lower_inclusive: lower,
            upper_inclusive: upper,
        }
    }

    pub fn contains(&self, note: u8) -> bool {
        self.lower_inclusive <= note && self.upper_inclusive >= note
    }
}

pub enum Status {
    Ok,
    Ended,
}

pub struct NoteEvent {
    kind: NoteKind,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum NoteKind {
    NoteOn(u8),
    NoteOff(u8),
}
