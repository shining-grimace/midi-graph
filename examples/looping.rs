extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, FileAssetLoader, Message, MessageSender,
    abstraction::NodeConfigData,
    effect::Fader,
    generator::{LfsrNoise, SawtoothWave},
    group::{Font, FontSource, RangeSource},
    midi::{CueData, Midi, MidiDataSource},
};
use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};

const MIDI_FILE: &'static str = "resources/LoopingMidi.mid";

const NOISE_CHANNEL: usize = 0;
const LEAD_CHANNEL: usize = 1;

const MIDI_NODE_ID: u64 = 100;
const FADER_NODE_ID: u64 = 101;

fn main() {
    let config = NodeConfigData(Box::new(Midi {
        node_id: Some(MIDI_NODE_ID),
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (
                NOISE_CHANNEL,
                NodeConfigData(Box::new(Font {
                    node_id: None,
                    config: FontSource::Ranges(vec![RangeSource {
                        source: NodeConfigData(Box::new(Fader {
                            node_id: Some(FADER_NODE_ID),
                            initial_volume: 0.0,
                            source: NodeConfigData(Box::new(LfsrNoise {
                                node_id: None,
                                balance: Balance::Left,
                                amplitude: 0.5,
                                inside_feedback: true,
                                note_for_16_shifts: 70,
                            })),
                        })),
                        lower: 0,
                        upper: 127,
                    }]),
                })),
            ),
            (
                LEAD_CHANNEL,
                NodeConfigData(Box::new(Font {
                    node_id: None,
                    config: FontSource::Ranges(vec![RangeSource {
                        source: NodeConfigData(Box::new(SawtoothWave {
                            node_id: None,
                            balance: Balance::Right,
                            amplitude: 0.5,
                        })),
                        lower: 0,
                        upper: 127,
                    }]),
                })),
            ),
        ]),
    }));
    let mut asset_loader = FileAssetLoader::default();
    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program_from_config(1, config, &mut asset_loader)
        .unwrap()
        .start(Some(1))
        .unwrap();
    let mut sender = mixer.get_event_sender();
    std::thread::spawn(move || {
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(FADER_NODE_ID),
            Event::Fade {
                from: 0.0,
                to: 1.0,
                seconds: 1.0,
            },
        );
        sleep(Duration::from_secs(12));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(MIDI_NODE_ID),
            Event::CueData(CueData::SeekWhenIdeal(1)),
        );
    });
    sleep(Duration::from_secs(30));
}

fn send_or_log(sender: &mut Arc<MessageSender>, target: EventTarget, event: Event) {
    let message = Message {
        target,
        data: event,
    };
    if let Err(error) = sender.send(message) {
        println!("Send error: {:?}", error);
    }
}
