use crate::AudioSource;
use cpal::Sample;

pub struct SquareWaveSource {
    period_time: usize,
    is_high: bool,
}

impl Default for SquareWaveSource {
    fn default() -> Self {
        Self {
            period_time: 0,
            is_high: false,
        }
    }
}

impl AudioSource for SquareWaveSource {
    fn is_completed(&self) -> bool {
        false
    }

    fn rewind(&mut self) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let size = buffer.len();
        for i in 0..size {
            self.period_time += 1;
            if self.period_time >= 32 {
                // Like 400 Hz at 48 kHz and 2 channels
                self.is_high = !self.is_high;
            }
            buffer[i] = match self.is_high {
                true => Sample::from_sample(0.5),
                false => Sample::from_sample(-0.5),
            };
        }
    }
}
