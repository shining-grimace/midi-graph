use crate::{Balance, Error, Event, EventTarget, Message, Node, consts, util};

pub struct SquareWaveSource {
    node_id: u64,
    is_on: bool,
    current_note: u8,
    balance: Balance,
    current_amplitude: f32,
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    peak_amplitude: f32,
    duty_cycle: f32,
}

impl SquareWaveSource {
    pub fn new(node_id: Option<u64>, balance: Balance, amplitude: f32, duty_cycle: f32) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            current_note: 0,
            balance,
            current_amplitude: 0.0,
            cycle_progress_samples: 0.0,
            period_samples_a440: consts::PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            peak_amplitude: amplitude,
            duty_cycle,
        }
    }
}

impl Node for SquareWaveSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let source = Self::new(
            Some(self.node_id),
            self.balance,
            self.peak_amplitude,
            self.duty_cycle,
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
        let pitch_period_samples = consts::PLAYBACK_SAMPLE_RATE as f32 / note_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_period_samples / self.period_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        let (write_left, write_right) = match self.balance {
            Balance::Both => (true, true),
            Balance::Left => (true, false),
            Balance::Right => (false, true),
        };
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress += 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            let amplitude = match duty > self.duty_cycle {
                true => self.current_amplitude,
                false => -self.current_amplitude,
            };
            if write_left {
                buffer[i] += amplitude;
            }
            if write_right {
                buffer[i + 1] += amplitude;
            }
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
    }

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User(
                "SquareWaveSource cannot have children".to_owned(),
            )),
        }
    }
}
