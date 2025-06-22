extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, FileAssetLoader, Message, MessageSender,
    abstraction::NodeConfigData,
    effect::{Fader, Lfo, ModulationProperty, Transition},
    generator::{SawtoothWave, SquareWave, TriangleWave},
    group::{Font, FontSource, Mixer, RangeSource},
};
use std::{sync::Arc, thread::sleep, time::Duration};

const LFO_NODE_ID: u64 = 100;
const FADER_NODE_ID: u64 = 101;
const TRANSITION_NODE_ID: u64 = 102;

fn main() {
    let triangle_unison = Lfo {
        node_id: Some(LFO_NODE_ID),
        source: NodeConfigData(Box::new(Mixer {
            node_id: None,
            balance: 0.375,
            sources: [
                NodeConfigData(Box::new(TriangleWave {
                    node_id: None,
                    balance: Balance::Both,
                    amplitude: 0.75,
                })),
                NodeConfigData(Box::new(SawtoothWave {
                    node_id: None,
                    balance: Balance::Both,
                    amplitude: 0.1625,
                })),
            ],
        })),
    };

    let square_source = SquareWave {
        node_id: None,
        balance: Balance::Both,
        amplitude: 0.375,
        duty_cycle: 0.25,
    };
    let transition = Transition {
        node_id: Some(TRANSITION_NODE_ID),
        source: NodeConfigData(Box::new(square_source)),
    };
    let fader = Fader {
        node_id: Some(FADER_NODE_ID),
        initial_volume: 0.0,
        source: NodeConfigData(Box::new(transition)),
    };
    let soundfont = Font {
        node_id: None,
        config: FontSource::Ranges(vec![
            RangeSource {
                lower: 0,
                upper: 70,
                source: NodeConfigData(Box::new(triangle_unison)),
            },
            RangeSource {
                lower: 71,
                upper: 255,
                source: NodeConfigData(Box::new(fader)),
            },
        ]),
    };
    let mixer = BaseMixer::builder(|_| {})
        .unwrap()
        .set_initial_program_from_config(1, NodeConfigData(Box::new(soundfont)), &FileAssetLoader)
        .unwrap()
        .start(Some(1))
        .expect("Could not open stream");
    let mut sender = mixer.get_event_sender();
    std::thread::spawn(move || {
        sleep(Duration::from_millis(50));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOn { note: 69, vel: 1.0 },
        );
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(LFO_NODE_ID),
            Event::Lfo {
                property: ModulationProperty::Volume,
                low: 0.0,
                high: 1.0,
                period_secs: 0.4,
                steps: 12,
            },
        );
        sleep(Duration::from_millis(2000));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(LFO_NODE_ID),
            Event::EndModulation,
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOff { note: 69, vel: 0.0 },
        );
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(FADER_NODE_ID),
            Event::Fade {
                from: 0.0,
                to: 1.0,
                seconds: 1.0,
            },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOn {
                note: 73,
                vel: 0.375,
            },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::SourceBalance(Balance::Left),
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOff { note: 73, vel: 0.0 },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOn {
                note: 74,
                vel: 0.75,
            },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::SourceBalance(Balance::Right),
        );
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(TRANSITION_NODE_ID),
            Event::Transition {
                property: ModulationProperty::PitchMultiplier,
                from: 1.0,
                to: 1.5,
                duration_secs: 0.2,
                steps: 20,
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOff { note: 74, vel: 0.0 },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOn {
                note: 71,
                vel: 0.375,
            },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::SourceBalance(Balance::Both),
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOff { note: 71, vel: 0.0 },
        );
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOn { note: 69, vel: 1.0 },
        );
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOff { note: 69, vel: 0.0 },
        );
    });
    sleep(Duration::from_secs(5));
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
