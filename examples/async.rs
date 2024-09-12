extern crate midi_graph;

use cpal::traits::StreamTrait;
use crossbeam_channel::Sender;
use midi_graph::{
    AsyncEventReceiver, BaseMixer, NoteEvent, NoteKind, NoteRange, SoundFontBuilder,
    SquareWaveSource,
};
use std::{thread::sleep, time::Duration};

fn main() {
    let square_font = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_inclusive_range(0, 50),
            Box::new(SquareWaveSource::new(0.125, 0.5)),
        )
        .unwrap()
        .add_range(
            NoteRange::new_inclusive_range(51, 255),
            Box::new(SquareWaveSource::new(0.125, 0.875)),
        )
        .unwrap()
        .build();
    let (mut sender, receiver) = AsyncEventReceiver::new(Box::new(square_font));
    let mixer = BaseMixer::from_source(Box::new(receiver));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::spawn(move || {
        sleep(Duration::from_millis(50));
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOn { note: 69, vel: 1.0 },
            },
        );
        sleep(Duration::from_millis(1500));
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOff { note: 69, vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOn {
                    note: 73,
                    vel: 0.375,
                },
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOff { note: 73, vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOn {
                    note: 74,
                    vel: 0.75,
                },
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOff { note: 74, vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOn {
                    note: 71,
                    vel: 0.375,
                },
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOff { note: 71, vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOn { note: 69, vel: 1.0 },
            },
        );
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            NoteEvent {
                kind: NoteKind::NoteOff { note: 69, vel: 0.0 },
            },
        );
    });
    sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}

fn send_or_log(sender: &mut Sender<NoteEvent>, event: NoteEvent) {
    if let Err(error) = sender.send(event) {
        println!("Send error: {:?}", error);
    }
}
