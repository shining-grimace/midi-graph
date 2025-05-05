extern crate midi_graph;

use crossbeam_channel::Sender;
use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, Message, NoteRange,
    effect::{AsyncEventReceiver, Fader},
    font::SoundFontBuilder,
    generator::{SawtoothWaveSource, SquareWaveSource, TriangleWaveSource},
    group::MixerSource,
};
use std::{thread::sleep, time::Duration};

const MIXER_NODE_ID: u64 = 100;
const FADER_NODE_ID: u64 = 101;

fn main() {
    let triangle_unison = MixerSource::new(
        Some(MIXER_NODE_ID),
        0.375,
        Box::new(TriangleWaveSource::new(None, Balance::Both, 0.75)),
        Box::new(SawtoothWaveSource::new(None, Balance::Both, 0.1625)),
    );

    let square_source = SquareWaveSource::new(None, Balance::Both, 0.375, 0.25);
    let fader = Fader::new(Some(FADER_NODE_ID), 0.0, Box::new(square_source));
    let soundfont = SoundFontBuilder::new(None)
        .add_range(
            NoteRange::new_inclusive_range(0, 70),
            Box::new(triangle_unison),
        )
        .unwrap()
        .add_range(NoteRange::new_inclusive_range(71, 255), Box::new(fader))
        .unwrap()
        .build();
    let (mut sender, receiver) = AsyncEventReceiver::new(None, Box::new(soundfont));
    let _mixer =
        BaseMixer::start_single_program(Box::new(receiver)).expect("Could not open stream");
    std::thread::spawn(move || {
        sleep(Duration::from_millis(50));
        send_or_log(
            &mut sender,
            EventTarget::Broadcast,
            Event::NoteOn { note: 69, vel: 1.0 },
        );
        for _ in 0..10 {
            sleep(Duration::from_millis(100));
            send_or_log(
                &mut sender,
                EventTarget::SpecificNode(MIXER_NODE_ID),
                Event::MixerBalance(0.625),
            );
            sleep(Duration::from_millis(100));
            send_or_log(
                &mut sender,
                EventTarget::SpecificNode(MIXER_NODE_ID),
                Event::MixerBalance(0.375),
            );
        }
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

fn send_or_log(sender: &mut Sender<Message>, target: EventTarget, event: Event) {
    let message = Message {
        target,
        data: event
    };
    if let Err(error) = sender.send(message) {
        println!("Send error: {:?}", error);
    }
}
