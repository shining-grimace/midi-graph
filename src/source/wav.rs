
use crate::AudioStreamer;
use wav::{header::Header, bit_depth::BitDepth};

pub struct WavAudio {
    position: usize,
    length: usize,
    data: Vec<f32>
}

impl WavAudio {
    pub fn new_from_data(_header: Header, data: BitDepth) -> Self {
        let data = match data {
            BitDepth::ThirtyTwoFloat(samples) => samples,
            _ => panic!("Non-f32 samples not currently supported")
        };
        let length = data.len();
        Self {
            position: 0,
            length,
            data
        }
    }
}

impl AudioStreamer for WavAudio {

    fn is_completed(&self) -> bool {
        false
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let size = buffer.len();
        let samples_remaining = self.length - self.position;
        if samples_remaining < size {
            buffer.copy_from_slice(&self.data[self.position .. (self.position + samples_remaining)]);
            &buffer[samples_remaining .. size].fill(0.0);
        } else {
            buffer.copy_from_slice(&self.data[self.position .. (self.position + size)]);
        }
    }
}
