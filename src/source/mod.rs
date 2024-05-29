pub mod midi;
pub mod square;
pub mod wav;

pub trait AudioSource {
    fn is_completed(&self) -> bool;
    fn fill_buffer(&mut self, buffer: &mut [f32]);
}
