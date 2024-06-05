use crate::AudioSource;

pub struct SquareWaveSource {
    cycle_progress: usize,
    period_length: usize,
    cycle_on_time: usize,
}

impl Default for SquareWaveSource {
    fn default() -> Self {
        Self {
            cycle_progress: 0,
            period_length: 48,
            cycle_on_time: 32,
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
            self.cycle_progress = (self.cycle_progress + 1) % self.period_length;
            buffer[i] = match self.cycle_progress >= self.cycle_on_time {
                true => 1.0,
                false => -1.0,
            };
        }
    }
}
