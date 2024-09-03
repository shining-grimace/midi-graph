use crate::{consts, util, BufferConsumer, NoteEvent, NoteKind};

pub struct LfsrNoiseSource {
    is_on: bool,
    current_note: u8,
    current_lfsr: u16,
    feedback_mask: u16,
    cycle_progress_samples: f32,
    cycle_samples_a440: f32,
    amplitude: f32,
}

impl LfsrNoiseSource {
    pub fn new(amplitude: f32, inside_feedback: bool, note_of_16_shifts: u8) -> Self {
        let feedback_mask = match inside_feedback {
            true => 0x4040,
            false => 0x4000,
        };
        let rotations_per_second_requested = util::frequency_of(note_of_16_shifts);
        let rotations_per_second_a440 = util::frequency_of(69);
        let shifts_per_rotation = 16.0;
        let samples_per_second = consts::PLAYBACK_SAMPLE_RATE as f32;
        let cycle_samples_a440 = samples_per_second
            / (shifts_per_rotation * rotations_per_second_a440)
            / (rotations_per_second_requested / rotations_per_second_a440);
        Self {
            is_on: false,
            current_note: 0,
            current_lfsr: 0x0001,
            feedback_mask,
            cycle_progress_samples: 0.0,
            cycle_samples_a440,
            amplitude,
        }
    }

    #[inline]
    fn value(&self) -> f32 {
        match self.current_lfsr & 0x0001 {
            0x0001 => self.amplitude,
            _ => -self.amplitude,
        }
    }

    fn shift(&mut self) {
        let feedback_bits = (self.current_lfsr & 0x0001) ^ ((self.current_lfsr & 0x0002) >> 1);
        let masked_feedback = feedback_bits * self.feedback_mask;
        self.current_lfsr = ((self.current_lfsr >> 1) & !masked_feedback) | masked_feedback;
    }
}

impl BufferConsumer for LfsrNoiseSource {
    fn set_note(&mut self, event: NoteEvent) {
        match event.kind {
            NoteKind::NoteOn(note) => {
                self.is_on = true;
                self.current_note = note;
            }
            NoteKind::NoteOff(note) => {
                if self.current_note != note {
                    return;
                }
                self.is_on = false;
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if !self.is_on {
            return;
        }
        let size = buffer.len();
        let note_frequency = util::frequency_of(self.current_note);
        let pitch_cycle_samples = consts::PLAYBACK_SAMPLE_RATE as f32 / note_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_cycle_samples / self.cycle_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        let mut current_value = self.value();
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress = stretched_progress + 1.0;
            if stretched_progress >= pitch_cycle_samples {
                stretched_progress -= pitch_cycle_samples;
                self.shift();
                current_value = self.value();
            }
            buffer[i] += current_value;
            buffer[i + 1] += current_value;
        }

        self.cycle_progress_samples =
            stretched_progress * self.cycle_samples_a440 / pitch_cycle_samples;
    }
}
