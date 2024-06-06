use crate::{constants::PLAYBACK_SAMPLE_RATE, AudioSource};

pub struct SquareWaveSource {
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    duty_cycle: f32,
}

impl Default for SquareWaveSource {
    fn default() -> Self {
        Self {
            cycle_progress_samples: 0.0,
            period_samples_a440: PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            duty_cycle: 0.75,
        }
    }
}

impl AudioSource for SquareWaveSource {
    fn is_completed(&self) -> bool {
        false
    }

    fn rewind(&mut self) {}

    fn fill_buffer(&mut self, relative_pitch: f32, buffer: &mut [f32]) {
        let size = buffer.len();
        let note_frequency = 440.0 * 2.0f32.powf(relative_pitch / 12.0);
        let pitch_period_samples = PLAYBACK_SAMPLE_RATE as f32 / note_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_period_samples / self.period_samples_a440;

        for i in 0..size {
            stretched_progress = stretched_progress + 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            buffer[i] = match duty > self.duty_cycle {
                true => 1.0,
                false => -1.0,
            };
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
    }
}
