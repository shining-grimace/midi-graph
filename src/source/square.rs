use crate::{config, util, AudioSource};

pub struct SquareWaveSource {
    is_on: bool,
    current_note: u8,
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    duty_cycle: f32,
}

impl Default for SquareWaveSource {
    fn default() -> Self {
        Self {
            is_on: false,
            current_note: 0,
            cycle_progress_samples: 0.0,
            period_samples_a440: config::PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            duty_cycle: 0.75,
        }
    }
}

impl AudioSource for SquareWaveSource {
    fn on_note_on(&mut self, key: u8) {
        self.is_on = true;
        self.current_note = key;
    }

    fn on_note_off(&mut self, key: u8) {
        if self.current_note != key {
            return;
        }
        self.is_on = false;
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if !self.is_on {
            return;
        }
        let size = buffer.len();
        let note_frequency = util::frequency_of(self.current_note);
        let pitch_period_samples = config::PLAYBACK_SAMPLE_RATE as f32 / note_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_period_samples / self.period_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % config::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(config::CHANNEL_COUNT, 2);

        for i in (0..size).step_by(config::CHANNEL_COUNT) {
            stretched_progress = stretched_progress + 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            let amplitude = match duty > self.duty_cycle {
                true => 0.25,
                false => -0.25,
            };
            buffer[i] += amplitude;
            buffer[i + 1] += amplitude;
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
    }
}
