use crate::{
    AssetLoader, Balance, Error, Event, EventTarget, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
    consts, util,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct LfsrNoise {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::source_balance")]
    pub balance: Balance,
    #[serde(default = "defaults::amplitude")]
    pub amplitude: f32,
    pub inside_feedback: bool,
    #[serde(default = "defaults::note_for_16_shifts")]
    pub note_for_16_shifts: u8,
}

impl LfsrNoise {
    pub fn stock(inside_feedback_mode: bool) -> NodeConfigData {
        NodeConfigData(Box::new(Self {
            node_id: defaults::none_id(),
            balance: Balance::Both,
            amplitude: defaults::amplitude(),
            inside_feedback: inside_feedback_mode,
            note_for_16_shifts: defaults::note_for_16_shifts(),
        }))
    }
}

impl NodeConfig for LfsrNoise {
    fn to_node(&self, _asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        Ok(Box::new(LfsrNoiseNode::new(
            self.node_id,
            self.balance,
            self.amplitude,
            self.inside_feedback,
            self.note_for_16_shifts,
        )))
    }

    fn clone_child_configs(&self) -> Option<Vec<crate::abstraction::NodeConfigData>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct LfsrNoiseNode {
    node_id: u64,
    is_on: bool,
    note_of_16_shifts: u8,
    current_note: u8,
    current_frequency: f32,
    balance: Balance,
    current_lfsr: u16,
    feedback_mask: u16,
    cycle_progress_samples: f32,
    cycle_samples_a440: f32,
    peak_amplitude: f32,
    note_velocity: f32,
    modulated_volume: f32,
}

impl LfsrNoiseNode {
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
            current_frequency: 10.0,
            balance,
            current_lfsr: 0x0001,
            feedback_mask,
            cycle_progress_samples: 0.0,
            cycle_samples_a440,
            peak_amplitude: amplitude,
            note_velocity: 1.0,
            modulated_volume: 1.0,
        }
    }

    #[inline]
    fn value(&self) -> f32 {
        let current_amplitude = self.peak_amplitude * self.note_velocity * self.modulated_volume;
        match self.current_lfsr & 0x0001 {
            0x0001 => current_amplitude,
            _ => -current_amplitude,
        }
    }

    fn shift(&mut self) {
        let feedback_bits = (self.current_lfsr & 0x0001) ^ ((self.current_lfsr & 0x0002) >> 1);
        let masked_feedback = feedback_bits * self.feedback_mask;
        self.current_lfsr = ((self.current_lfsr >> 1) & !masked_feedback) | masked_feedback;
    }
}

impl Node for LfsrNoiseNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
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

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOff { note, .. } => {
                if note == self.current_note || event.target == EventTarget::Broadcast {
                    self.is_on = false;
                }
            }
            Event::NoteOn { note, vel } => {
                self.is_on = true;
                self.current_note = note;
                self.current_frequency = util::frequency_of(note);
                self.note_velocity = vel;
            }
            Event::PitchMultiplier(multiplier) => {
                self.current_frequency = multiplier * util::frequency_of(self.current_note);
            }
            Event::SourceBalance(balance) => {
                self.balance = balance;
            }
            Event::Volume(volume) => {
                self.modulated_volume = volume;
            }
            _ => {}
        }
        true
    }

    fn propagate(&mut self, _event: &Message) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if !self.is_on {
            return;
        }
        let size = buffer.len();
        let pitch_cycle_samples = consts::PLAYBACK_SAMPLE_RATE as f32 / self.current_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_cycle_samples / self.cycle_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        let mut current_amplitude = self.value();
        let (left_amplitude, right_amplitude) = match self.balance {
            Balance::Both => (1.0, 1.0),
            Balance::Left => (1.0, 0.0),
            Balance::Right => (0.0, 1.0),
            Balance::Pan(pan) => (1.0 - pan, pan),
        };
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress += 1.0;
            if stretched_progress >= pitch_cycle_samples {
                stretched_progress -= pitch_cycle_samples;
                self.shift();
                current_amplitude = self.value();
            }
            buffer[i] += left_amplitude * current_amplitude;
            buffer[i + 1] += right_amplitude * current_amplitude;
        }

        self.cycle_progress_samples =
            stretched_progress * self.cycle_samples_a440 / pitch_cycle_samples;
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User(
                "LfsrNoiseSource cannot have children".to_owned(),
            )),
        }
    }
}
