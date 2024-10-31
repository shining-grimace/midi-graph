extern crate midi_graph;

use cpal::traits::StreamTrait;
use crossbeam_channel::Sender;
use midi_graph::{
    AsyncEventReceiver, BaseMixer, Config, FontSource, MidiDataSource, MidiSource,
    NodeControlEvent, NodeEvent, RangeSource, SoundSource,
};
use std::{collections::HashMap, thread::sleep, time::Duration};

const MIDI_FILE: &'static str = "resources/LoopingMidi.mid";

const NOISE_CHANNEL: usize = 0;
const LEAD_CHANNEL: usize = 1;

const FADER_NODE_ID: u64 = 100;

fn main() {
    let config = Config {
        midi: MidiDataSource::FilePath(MIDI_FILE.to_owned()),
        channels: HashMap::from([
            (
                NOISE_CHANNEL,
                FontSource::Ranges(vec![RangeSource {
                    source: SoundSource::Fader {
                        node_id: Some(FADER_NODE_ID),
                        initial_volume: 0.0,
                        source: Box::new(SoundSource::LfsrNoise {
                            node_id: None,
                            amplitude: 0.5,
                            inside_feedback: true,
                            note_for_16_shifts: 70,
                        }),
                    },
                    lower: 0,
                    upper: 127,
                }]),
            ),
            (
                LEAD_CHANNEL,
                FontSource::Ranges(vec![RangeSource {
                    source: SoundSource::SawtoothWave {
                        node_id: None,
                        amplitude: 0.5,
                    },
                    lower: 0,
                    upper: 127,
                }]),
            ),
        ]),
    };
    let main_tree = MidiSource::from_config(config).unwrap();
    let (mut sender, receiver) = AsyncEventReceiver::new(None, Box::new(main_tree));
    let mixer = BaseMixer::from_consumer(Box::new(receiver));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::spawn(move || {
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            &NodeEvent::NodeControl {
                node_id: FADER_NODE_ID,
                event: NodeControlEvent::Fade {
                    from: 0.0,
                    to: 1.0,
                    seconds: 1.0,
                },
            },
        );
    });
    sleep(Duration::from_secs(20));
    stream.pause().expect("Could not pause the stream");
}

fn send_or_log(sender: &mut Sender<NodeEvent>, event: &NodeEvent) {
    if let Err(error) = sender.send(event.clone()) {
        println!("Send error: {:?}", error);
    }
}
