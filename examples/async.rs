extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, Message, MessageSender, NoteRange,
    effect::{Fader, Lfo, ModulationProperty, TransitionEnvelope},
    generator::{SawtoothWaveSource, SquareWaveSource, TriangleWaveSource},
    group::{MixerSource, SoundFontBuilder},
};
use std::{sync::Arc, thread::sleep, time::Duration};

const LFO_NODE_ID: u64 = 100;
const FADER_NODE_ID: u64 = 101;
const TRANSITION_NODE_ID: u64 = 102;

fn main() {
    let triangle_unison = Lfo::new(
        Some(LFO_NODE_ID),
        Box::new(MixerSource::new(
            None,
            0.375,
            Box::new(TriangleWaveSource::new(None, Balance::Both, 0.75)),
            Box::new(SawtoothWaveSource::new(None, Balance::Both, 0.1625)),
        )),
    )
    .unwrap();

    let square_source = SquareWaveSource::new(None, Balance::Both, 0.375, 0.25);
    let transition =
        TransitionEnvelope::new(Some(TRANSITION_NODE_ID), Box::new(square_source)).unwrap();
    let fader = Fader::new(Some(FADER_NODE_ID), 0.0, Box::new(transition));
    let soundfont = SoundFontBuilder::new(None)
        .add_range(
            NoteRange::new_inclusive_range(0, 70),
            Box::new(triangle_unison),
        )
        .unwrap()
        .add_range(NoteRange::new_inclusive_range(71, 255), Box::new(fader))
        .unwrap()
        .build();
    let mixer =
        BaseMixer::start_single_program(Box::new(soundfont)).expect("Could not open stream");
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
