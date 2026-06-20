extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, EventTiming, Message, MessageSender,
    effect::AdsrEnvelopeNode, generator::TriangleWaveNode, group::PolyphonyNode,
};
use std::{sync::Arc, thread::sleep, time::Duration};

const POLYPHONY_NODE_ID: u64 = 100;

fn main() {
    let inner = AdsrEnvelopeNode::from_parameters(
        None,
        0.05,
        0.2,
        0.8,
        0.2,
        Box::new(TriangleWaveNode::new(None, Balance::Both, 0.75)),
    );
    let polyphony = PolyphonyNode::new(Some(POLYPHONY_NODE_ID), 6, Box::new(inner)).unwrap();
    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(polyphony))
        .start(Some(1))
        .unwrap();
    let mut sender = mixer.get_event_sender();
    let absolute_frame = sender.current_rendering_absolute_frame();
    send_after(
        &mut sender,
        EventTarget::SpecificNode(POLYPHONY_NODE_ID),
        Event::NoteOn { note: 69, vel: 1.0 },
        absolute_frame,
        0.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(POLYPHONY_NODE_ID),
        Event::NoteOn { note: 72, vel: 1.0 },
        absolute_frame,
        0.55,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(POLYPHONY_NODE_ID),
        Event::NoteOn { note: 75, vel: 1.0 },
        absolute_frame,
        1.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(POLYPHONY_NODE_ID),
        Event::NoteOff { note: 69, vel: 1.0 },
        absolute_frame,
        2.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(POLYPHONY_NODE_ID),
        Event::NoteOff { note: 75, vel: 1.0 },
        absolute_frame,
        3.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(POLYPHONY_NODE_ID),
        Event::NoteOff { note: 72, vel: 1.0 },
        absolute_frame,
        3.55,
    );
    sleep(Duration::from_secs(4));
}

fn send_after(
    sender: &mut Arc<MessageSender>,
    target: EventTarget,
    event: Event,
    absolute_frame: u64,
    seconds: f32,
) {
    let message = Message {
        target,
        data: event,
        timing: EventTiming::after_seconds(absolute_frame, seconds),
    };
    if let Err(error) = sender.send(message) {
        println!("Send error: {:?}", error);
    }
}
