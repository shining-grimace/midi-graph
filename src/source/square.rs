use crate::{consts, util, BufferConsumer, Error, NoteEvent, NoteKind, Status};

pub struct SquareWaveSource {
    is_on: bool,
    current_note: u8,
    current_amplitude: f32,
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    peak_amplitude: f32,
    duty_cycle: f32,
}

impl SquareWaveSource {
    pub fn new(amplitude: f32, duty_cycle: f32) -> Self {
        Self {
            is_on: false,
            current_note: 0,
            current_amplitude: 0.0,
            cycle_progress_samples: 0.0,
            period_samples_a440: consts::PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            peak_amplitude: amplitude,
            duty_cycle,
        }
    }
}

impl BufferConsumer for SquareWaveSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        let source = Self::new(self.peak_amplitude, self.duty_cycle);
        Ok(Box::new(source))
    }

    fn set_note(&mut self, event: NoteEvent) {
        match event.kind {
            NoteKind::NoteOn { note, vel } => {
                self.is_on = true;
                self.current_note = note;
                self.current_amplitude = self.peak_amplitude * vel;
            }
            NoteKind::NoteOff { note, vel: _ } => {
                if self.current_note != note {
                    return;
                }
                self.is_on = false;
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        if !self.is_on {
            return Status::Ended;
        }
        let size = buffer.len();
        let note_frequency = util::frequency_of(self.current_note);
        let pitch_period_samples = consts::PLAYBACK_SAMPLE_RATE as f32 / note_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_period_samples / self.period_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress = stretched_progress + 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            let amplitude = match duty > self.duty_cycle {
                true => self.current_amplitude,
                false => -self.current_amplitude,
            };
            buffer[i] += amplitude;
            buffer[i + 1] += amplitude;
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
        Status::Ok
    }
}
