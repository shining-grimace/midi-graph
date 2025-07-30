extern crate midi_graph;

use midi_graph::{
    AssetLoader, BaseMixer, Error, Event, EventTarget, FileAssetLoader, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
    consts,
    group::Subtree,
    midi::{Midi, MidiDataSource},
    util,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const CHANNEL_0: usize = 0;
const CHANNEL_1: usize = 1;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const JSON_SUBTREE_FILE: &'static str = "resources/custom-example-subtree.json";

fn main() {
    let subtree_config = Subtree::as_path(JSON_SUBTREE_FILE);
    let config = NodeConfigData(Box::new(Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (CHANNEL_0, NodeConfigData(Box::new(subtree_config))),
            (
                CHANNEL_1,
                NodeConfigData(Box::new(SineWave {
                    node_id: None,
                    amplitude: 0.5,
                })),
            ),
        ]),
    }));
    let mut asset_loader = FileAssetLoader::default();
    let _mixer = BaseMixer::builder_with_custom_registry(|registry| {
        registry.register_node_type::<SineWave>("SineWave");
    })
    .unwrap()
    .set_initial_program_from_config(1, config, &mut asset_loader)
    .unwrap()
    .start(Some(1))
    .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}

#[derive(Debug, Deserialize, Clone)]
pub struct SineWave {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::amplitude")]
    pub amplitude: f32,
}

impl NodeConfig for SineWave {
    fn to_node(&self, _asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        Ok(Box::new(SineWaveNode::new(self.node_id, self.amplitude)))
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct SineWaveNode {
    node_id: u64,
    is_on: bool,
    current_note: u8,
    current_frequency: f32,
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    peak_amplitude: f32,
    note_velocity: f32,
}

impl SineWaveNode {
    pub fn new(node_id: Option<u64>, amplitude: f32) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            current_note: 0,
            current_frequency: 1.0,
            cycle_progress_samples: 0.0,
            period_samples_a440: consts::PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            peak_amplitude: amplitude,
            note_velocity: 1.0,
        }
    }
}

impl Node for SineWaveNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let source = Self::new(Some(self.node_id), self.peak_amplitude);
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
                self.cycle_progress_samples = 0.0;
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

        let current_amplitude = self.peak_amplitude * self.note_velocity;
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress += 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            let amplitude = current_amplitude * (duty * 2.0 * std::f32::consts::PI).sin();
            buffer[i] += amplitude;
            buffer[i + 1] += amplitude;
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User(
                "SineWaveSource cannot have children".to_owned(),
            )),
        }
    }
}
