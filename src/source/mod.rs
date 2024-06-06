pub mod midi;
pub mod square;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

pub trait AudioSource {
    fn is_completed(&self) -> bool;
    fn rewind(&mut self);
    fn fill_buffer(&mut self, relative_pitch: f32, buffer: &mut [f32]);
}
