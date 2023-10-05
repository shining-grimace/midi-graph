
use crate::AudioStreamer;
use cpal::Sample;

pub struct SquareAudio {
    period_time: usize,
    is_high: bool
}

impl Default for SquareAudio {
    fn default() -> Self {
        Self { period_time: 0, is_high: false }
    }
}

impl AudioStreamer for SquareAudio {
    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let length = buffer.len();
        for i in 0..length {
            self.period_time += 1;
            if self.period_time >= 32 { // Like 400 Hz at 48 kHz and 2 channels
                self.is_high = !self.is_high;
            }
            buffer[i] = match self.is_high {
                true => Sample::from_sample(0.5),
                false => Sample::from_sample(-0.5)
            };
        }
    }
}
