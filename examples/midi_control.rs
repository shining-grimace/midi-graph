extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, FileAssetLoader, Message, MidiPlaybackState,
    abstraction::{ChildConfig, NodeConfig},
    generator::{LfsrNoise, SquareWave, TriangleWave},
    group::{Font, FontSource, RangeSource},
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

const MIDI_0_FILE: &'static str = "resources/sample-in-c.mid";

const NODE_ID: u64 = 100;

const PROGRAM_0: usize = 0;

fn main() {
    fn square_font() -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(Font {
            node_id: None,
            config: FontSource::Ranges(vec![RangeSource {
                source: ChildConfig(Box::new(SquareWave {
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
                source: ChildConfig(Box::new(TriangleWave {
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
                source: ChildConfig(Box::new(LfsrNoise {
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
        node_id: Some(NODE_ID),
        source: MidiDataSource::FilePath {
            path: MIDI_0_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (0, ChildConfig(triangle_font())),
            (1, ChildConfig(square_font())),
            (2, ChildConfig(noise_font())),
        ]),
    }
    .to_node(&mut asset_loader)
    .unwrap();

    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .store_program(PROGRAM_0, program_0)
        .start(Some(PROGRAM_0))
        .unwrap();
    let sender = mixer.get_event_sender();
    std::thread::sleep(Duration::from_secs(2));
    sender
        .send(Message {
            target: EventTarget::SpecificNode(NODE_ID),
            data: Event::MidiPlayback(MidiPlaybackState::Paused),
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
    sender
        .send(Message {
            target: EventTarget::SpecificNode(NODE_ID),
            data: Event::MidiPlayback(MidiPlaybackState::Playing),
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
}
