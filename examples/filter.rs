extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, FileAssetLoader, IirFilter, Message,
    abstraction::NodeConfigData,
    effect::Filter,
    generator::{LfsrNoise, SawtoothWave, SquareWave, TriangleWave},
    group::{Font, FontSource, Mixer, Polyphony, RangeSource},
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const TRIANGLE_CHANNEL: usize = 0;
const SQUARE_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

const FILTER_NODE_ID: u64 = 1;

fn main() {
    let triangle_unison = Polyphony {
        node_id: None,
        max_voices: 4,
        source: NodeConfigData(Box::new(Mixer {
            node_id: None,
            balance: 0.5,
            sources: [
                NodeConfigData(Box::new(TriangleWave {
                    node_id: None,
                    balance: Balance::Left,
                    amplitude: 0.75,
                })),
                NodeConfigData(Box::new(SawtoothWave {
                    node_id: None,
                    balance: Balance::Right,
                    amplitude: 0.1875,
                })),
            ],
        })),
    };
    let triangle_font = Font {
        node_id: None,
        config: FontSource::Ranges(vec![RangeSource {
            source: NodeConfigData(Box::new(triangle_unison)),
            lower: 0,
            upper: 127,
        }]),
    };
    let square_node = Filter {
        node_id: Some(FILTER_NODE_ID),
        filter: Some((IirFilter::LowPass, 1000.0)),
        source: NodeConfigData(Box::new(Polyphony {
            node_id: None,
            max_voices: 4,
            source: NodeConfigData(Box::new(SquareWave {
                node_id: None,
                balance: Balance::Both,
                amplitude: 0.125,
                duty_cycle: 0.5,
            })),
        })),
    };
    let noise_font = Font {
        node_id: None,
        config: FontSource::Ranges(vec![RangeSource {
            source: NodeConfigData(Box::new(LfsrNoise {
                node_id: None,
                balance: Balance::Both,
                amplitude: 0.25,
                inside_feedback: false,
                note_for_16_shifts: 50,
            })),
            lower: 0,
            upper: 127,
        }]),
    };
    let mut asset_loader = FileAssetLoader::default();
    let midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (TRIANGLE_CHANNEL, NodeConfigData(Box::new(triangle_font))),
            (SQUARE_CHANNEL, NodeConfigData(Box::new(square_node))),
            (NOISE_CHANNEL, NodeConfigData(Box::new(noise_font))),
        ]),
    };
    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program_from_config(1, NodeConfigData(Box::new(midi)), &mut asset_loader)
        .unwrap()
        .start(Some(1))
        .unwrap();
    let sender = mixer.get_event_sender();
    std::thread::sleep(Duration::from_secs(8));
    sender
        .send(Message {
            target: EventTarget::SpecificNode(FILTER_NODE_ID),
            data: Event::Filter {
                filter: IirFilter::HighPass,
                cutoff_frequency: 4000.0,
            },
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(4));
    sender
        .send(Message {
            target: EventTarget::SpecificNode(FILTER_NODE_ID),
            data: Event::FilterFrequencyShift(-3000.0),
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(4));
}
