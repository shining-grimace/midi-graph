pub mod midi;
pub mod square;
pub mod wav;

#[cfg(debug_assertions)]
pub mod log;

pub trait AudioSource {
    fn is_completed(&self) -> bool;
    fn rewind(&mut self);
    fn fill_buffer(&mut self, buffer: &mut [f32]);
}
