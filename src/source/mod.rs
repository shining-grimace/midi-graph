
pub mod square;

pub trait AudioStreamer {
    fn fill_buffer(&mut self, buffer: &mut [f32]);
}
