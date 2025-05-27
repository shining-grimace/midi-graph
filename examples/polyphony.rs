extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, Message, MessageSender, effect::AdsrEnvelope,
    generator::TriangleWaveSource, group::Polyphony,
};
use std::{sync::Arc, thread::sleep, time::Duration};

const POLYPHONY_NODE_ID: u64 = 100;

fn main() {
    let inner = AdsrEnvelope::from_parameters(
        None,
        0.05,
        0.2,
        0.8,
        0.2,
        Box::new(TriangleWaveSource::new(None, Balance::Both, 0.75)),
    );
    let polyphony = Polyphony::new(Some(POLYPHONY_NODE_ID), 6, Box::new(inner)).unwrap();
    let mixer =
        BaseMixer::start_single_program(Box::new(polyphony)).expect("Could not open stream");
    let mut sender = mixer.get_event_sender();
    std::thread::spawn(move || {
        sleep(Duration::from_millis(50));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(POLYPHONY_NODE_ID),
            Event::NoteOn { note: 69, vel: 1.0 },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(POLYPHONY_NODE_ID),
            Event::NoteOn { note: 72, vel: 1.0 },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(POLYPHONY_NODE_ID),
            Event::NoteOn { note: 75, vel: 1.0 },
        );
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(POLYPHONY_NODE_ID),
            Event::NoteOff { note: 69, vel: 1.0 },
        );
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(POLYPHONY_NODE_ID),
            Event::NoteOff { note: 75, vel: 1.0 },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            EventTarget::SpecificNode(POLYPHONY_NODE_ID),
            Event::NoteOff { note: 72, vel: 1.0 },
        );
    });
    sleep(Duration::from_secs(4));
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
