extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, EventTiming, Message, MessageSender,
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

    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(lfo_square))
        .start(Some(1))
        .unwrap();
    let mut sender = mixer.get_event_sender();
    let absolute_time = sender.current_rendering_absolute_frame();
    send_after(
        &mut sender,
        EventTarget::Broadcast,
        Event::NoteOn { note: 69, vel: 1.0 },
        absolute_time,
        0.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(LFO_NODE_ID),
        Event::Lfo {
            property: ModulationProperty::Volume,
            low: 0.0,
            high: 1.0,
            period_secs: 0.4,
            steps: 12,
        },
        absolute_time,
        0.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(LFO_NODE_ID),
        Event::EndModulation,
        absolute_time,
        1.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(LFO_NODE_ID),
        Event::Lfo {
            property: ModulationProperty::Pan,
            low: 0.0,
            high: 1.0,
            period_secs: 0.4,
            steps: 12,
        },
        absolute_time,
        1.55,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(LFO_NODE_ID),
        Event::EndModulation,
        absolute_time,
        2.55,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(LFO_NODE_ID),
        Event::Lfo {
            property: ModulationProperty::PitchMultiplier,
            low: 0.875,
            high: 1.0,
            period_secs: 0.4,
            steps: 12,
        },
        absolute_time,
        3.05,
    );
    send_after(
        &mut sender,
        EventTarget::SpecificNode(LFO_NODE_ID),
        Event::EndModulation,
        absolute_time,
        4.05,
    );
    send_after(
        &mut sender,
        EventTarget::Broadcast,
        Event::NoteOff { note: 69, vel: 0.0 },
        absolute_time,
        4.55,
    );
    sleep(Duration::from_secs(5));
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
