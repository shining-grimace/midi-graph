use crate::{
    AssetLoader, Balance, Error, Event, EventTarget, GraphNode, Message, Node,
    abstraction::{ChildConfig, NodeConfig, defaults},
    consts, util,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TriangleWave {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::source_balance")]
    pub balance: Balance,
    #[serde(default = "defaults::amplitude")]
    pub amplitude: f32,
}

impl TriangleWave {
    pub fn stock() -> ChildConfig {
        ChildConfig(Box::new(Self {
            node_id: defaults::none_id(),
            balance: Balance::Both,
            amplitude: defaults::amplitude(),
        }))
    }
}

impl NodeConfig for TriangleWave {
    fn to_node(&self, _asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        Ok(Box::new(TriangleWaveNode::new(
            self.node_id,
            self.balance,
            self.amplitude,
        )))
    }

    fn clone_child_configs(&self) -> Option<Vec<crate::abstraction::ChildConfig>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct TriangleWaveNode {
    node_id: u64,
    is_on: bool,
    current_note: u8,
    current_frequency: f32,
    balance: Balance,
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    peak_amplitude: f32,
    note_velocity: f32,
    modulated_volume: f32,
}

impl TriangleWaveNode {
    pub fn new(node_id: Option<u64>, balance: Balance, amplitude: f32) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            current_note: 0,
            current_frequency: 10.0,
            balance,
            cycle_progress_samples: 0.0,
            period_samples_a440: consts::PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            peak_amplitude: amplitude,
            note_velocity: 1.0,
            modulated_volume: 1.0,
        }
    }
}

impl Node for TriangleWaveNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        Ok(Box::new(Self::new(
            Some(self.node_id),
            self.balance,
            self.peak_amplitude,
        )))
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
        let pitch_period_samples = consts::PLAYBACK_SAMPLE_RATE as f32 / self.current_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_period_samples / self.period_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        let current_amplitude = self.peak_amplitude * self.note_velocity * self.modulated_volume;
        let (left_amplitude, right_amplitude) = match self.balance {
            Balance::Both => (1.0, 1.0),
            Balance::Left => (1.0, 0.0),
            Balance::Right => (0.0, 1.0),
            Balance::Pan(pan) => (1.0 - pan, pan),
        };
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress += 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            let amplitude = match duty > 0.5 {
                true => current_amplitude * (3.0 - 4.0 * duty),
                false => current_amplitude * (4.0 * duty - 1.0),
            };
            buffer[i] += left_amplitude * amplitude;
            buffer[i + 1] += right_amplitude * amplitude;
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User(
                "TriangleWaveSource cannot have children".to_owned(),
            )),
        }
    }
}
