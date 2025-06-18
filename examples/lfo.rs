extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, FileAssetLoader, Message, MessageSender,
    effect::{LfoNode, ModulationProperty},
    generator::SquareWaveNode,
};
use std::{sync::Arc, thread::sleep, time::Duration};

const LFO_NODE_ID: u64 = 100;

fn main() {
    let lfo_square = LfoNode::new(
        Some(LFO_NODE_ID),
        Box::new(SquareWaveNode::new(None, Balance::Both, 0.375, 0.25)),
    )
    .unwrap();

    let mixer = BaseMixer::builder(FileAssetLoader, |_| {})
        .unwrap()
        .set_initial_program(1, Box::new(lfo_square))
        .start(Some(1))
        .unwrap();
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
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(LFO_NODE_ID),
            Event::EndModulation,
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(LFO_NODE_ID),
            Event::Lfo {
                property: ModulationProperty::Pan,
                low: 0.0,
                high: 1.0,
                period_secs: 0.4,
                steps: 12,
            },
        );
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(LFO_NODE_ID),
            Event::EndModulation,
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(LFO_NODE_ID),
            Event::Lfo {
                property: ModulationProperty::PitchMultiplier,
                low: 0.875,
                high: 1.0,
                period_secs: 0.4,
                steps: 12,
            },
        );
        sleep(Duration::from_millis(1000));
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
