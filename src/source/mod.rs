pub mod midi;
pub mod square;
pub mod util;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

pub trait AudioSource {
    fn on_note_on(&mut self, key: u8);
    fn on_note_off(&mut self, key: u8);
    fn fill_buffer(&mut self, key: u8, buffer: &mut [f32]);
}
