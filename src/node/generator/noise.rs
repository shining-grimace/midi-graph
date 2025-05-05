use crate::{Balance, Error, Event, EventTarget, Message, Node, consts, util};

pub struct LfsrNoiseSource {
    node_id: u64,
    is_on: bool,
    note_of_16_shifts: u8,
    current_note: u8,
    balance: Balance,
    current_amplitude: f32,
    current_lfsr: u16,
    feedback_mask: u16,
    cycle_progress_samples: f32,
    cycle_samples_a440: f32,
    peak_amplitude: f32,
}

impl LfsrNoiseSource {
    pub fn new(
        node_id: Option<u64>,
        balance: Balance,
        amplitude: f32,
        inside_feedback: bool,
        note_of_16_shifts: u8,
    ) -> Self {
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
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            note_of_16_shifts,
            current_note: 0,
            balance,
            current_amplitude: 0.0,
            current_lfsr: 0x0001,
            feedback_mask,
            cycle_progress_samples: 0.0,
            cycle_samples_a440,
            peak_amplitude: amplitude,
        }
    }

    #[inline]
    fn value(&self) -> f32 {
        match self.current_lfsr & 0x0001 {
            0x0001 => self.current_amplitude,
            _ => -self.current_amplitude,
        }
    }

    fn shift(&mut self) {
        let feedback_bits = (self.current_lfsr & 0x0001) ^ ((self.current_lfsr & 0x0002) >> 1);
        let masked_feedback = feedback_bits * self.feedback_mask;
        self.current_lfsr = ((self.current_lfsr >> 1) & !masked_feedback) | masked_feedback;
    }
}

impl Node for LfsrNoiseSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let inside_feedback = match self.feedback_mask {
            0x4040 => true,
            0x4000 => false,
            _ => {
                return Err(Error::Internal(format!(
                    "MidiGraph: Unexpected feedback mask {}",
                    self.feedback_mask
                )));
            }
        };
        let source = Self::new(
            Some(self.node_id),
            self.balance,
            self.peak_amplitude,
            inside_feedback,
            self.note_of_16_shifts,
        );
        Ok(Box::new(source))
    }

    fn on_event(&mut self, event: &Message) {
        if !event.target.influences(self.node_id) {
            return;
        }
        match event.data {
            Event::NoteOff { note, .. } => {
                if note == self.current_note || event.target == EventTarget::Broadcast {
                    self.is_on = false;
                }
            }
            Event::NoteOn { note, vel } => {
                self.is_on = true;
                self.current_note = note;
                self.current_amplitude = self.peak_amplitude * vel;
            }
            Event::SourceBalance(balance) => {
                self.balance = balance;
            }
            Event::Volume(volume) => {
                self.peak_amplitude = volume;
            }
            _ => {}
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

        let mut amplitude = self.value();
        let (write_left, write_right) = match self.balance {
            Balance::Both => (true, true),
            Balance::Left => (true, false),
            Balance::Right => (false, true),
        };
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress += 1.0;
            if stretched_progress >= pitch_cycle_samples {
                stretched_progress -= pitch_cycle_samples;
                self.shift();
                amplitude = self.value();
            }
            if write_left {
                buffer[i] += amplitude;
            }
            if write_right {
                buffer[i + 1] += amplitude;
            }
        }

        self.cycle_progress_samples =
            stretched_progress * self.cycle_samples_a440 / pitch_cycle_samples;
    }

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User(
                "LfsrNoiseSource cannot have children".to_owned(),
            )),
        }
    }
}
