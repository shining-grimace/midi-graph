pub mod font;
pub mod midi;
pub mod square;
pub mod util;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

pub trait BufferConsumer {
    fn set_note(&mut self, event: NoteEvent);
    fn fill_buffer(&mut self, buffer: &mut [f32]);
}

pub trait NoteConsumer {
    fn restart_with_event(&mut self, event: NoteEvent);
    fn fill_buffer(&mut self, buffer: &mut [f32]);
}

pub enum NoteEvent {
    NoteOn(u8),
    NoteOff(u8),
}
