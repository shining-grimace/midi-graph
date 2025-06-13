extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, FileGraphLoader, GraphLoader, Message, MessageSender,
    midi::CueData,
    serialize::{FontSource, MidiDataSource, RangeSource, SoundSource},
};
use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};

const MIDI_FILE: &'static str = "resources/LoopingMidi.mid";

const NOISE_CHANNEL: usize = 0;
const LEAD_CHANNEL: usize = 1;

const MIDI_NODE_ID: u64 = 100;
const FADER_NODE_ID: u64 = 101;

fn main() {
    let config = Box::new(SoundSource::Midi {
        node_id: Some(MIDI_NODE_ID),
        source: MidiDataSource::FilePath(MIDI_FILE.to_owned()),
        channels: HashMap::from([
            (
                NOISE_CHANNEL,
                SoundSource::Font {
                    node_id: None,
                    config: FontSource::Ranges(vec![RangeSource {
                        source: SoundSource::Fader {
                            node_id: Some(FADER_NODE_ID),
                            initial_volume: 0.0,
                            source: Box::new(SoundSource::LfsrNoise {
                                node_id: None,
                                balance: Balance::Left,
                                amplitude: 0.5,
                                inside_feedback: true,
                                note_for_16_shifts: 70,
                            }),
                        },
                        lower: 0,
                        upper: 127,
                    }]),
                },
            ),
            (
                LEAD_CHANNEL,
                SoundSource::Font {
                    node_id: None,
                    config: FontSource::Ranges(vec![RangeSource {
                        source: SoundSource::SawtoothWave {
                            node_id: None,
                            balance: Balance::Right,
                            amplitude: 0.5,
                        },
                        lower: 0,
                        upper: 127,
                    }]),
                },
            ),
        ]),
    });
    let loader = FileGraphLoader::default();
    let source = loader
        .load_source_with_dependencies(&config)
        .expect("Could not create MIDI");
    let mixer = BaseMixer::start_single_program(source).expect("Could not start stream");
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
