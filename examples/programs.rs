extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, FileAssetLoader,
    abstraction::{NodeConfig, NodeConfigData},
    generator::{LfsrNoise, SquareWave, TriangleWave},
    group::{Font, FontSource, RangeSource},
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

const MIDI_0_FILE: &'static str = "resources/sample-in-c.mid";
const MIDI_1_FILE: &'static str = "resources/LoopingMidi.mid";

const PROGRAM_0: usize = 0;
const PROGRAM_1: usize = 7;

fn main() {
    fn square_font() -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(Font {
            node_id: None,
            config: FontSource::Ranges(vec![RangeSource {
                source: NodeConfigData(Box::new(SquareWave {
                    node_id: None,
                    balance: Balance::Right,
                    amplitude: 0.125,
                    duty_cycle: 0.0625,
                })),
                lower: 0,
                upper: 127,
            }]),
        })
    }

    fn triangle_font() -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(Font {
            node_id: None,
            config: FontSource::Ranges(vec![RangeSource {
                source: NodeConfigData(Box::new(TriangleWave {
                    node_id: None,
                    balance: Balance::Both,
                    amplitude: 1.0,
                })),
                lower: 0,
                upper: 127,
            }]),
        })
    }

    fn noise_font() -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(Font {
            node_id: None,
            config: FontSource::Ranges(vec![RangeSource {
                source: NodeConfigData(Box::new(LfsrNoise {
                    node_id: None,
                    balance: Balance::Left,
                    amplitude: 0.25,
                    inside_feedback: false,
                    note_for_16_shifts: 50,
                })),
                lower: 0,
                upper: 127,
            }]),
        })
    }

    let mut asset_loader = FileAssetLoader::default();
    let program_0 = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_0_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (0, NodeConfigData(triangle_font())),
            (1, NodeConfigData(square_font())),
            (2, NodeConfigData(noise_font())),
        ]),
    }
    .to_node(&mut asset_loader)
    .unwrap();
    let program_1 = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_1_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (0, NodeConfigData(noise_font())),
            (1, NodeConfigData(square_font())),
        ]),
    }
    .to_node(&mut asset_loader)
    .unwrap();

    let mut mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .store_program(PROGRAM_0, program_0)
        .store_program(PROGRAM_1, program_1)
        .start(None)
        .unwrap();
    std::thread::sleep(Duration::from_secs(1));
    mixer.change_program(PROGRAM_0).unwrap();
    std::thread::sleep(Duration::from_secs(6));
    mixer.change_program(PROGRAM_1).unwrap();
    std::thread::sleep(Duration::from_secs(6));
}
