extern crate midi_graph;

use cpal::traits::StreamTrait;
use crossbeam_channel::Sender;
use midi_graph::{
    AsyncEventReceiver, BaseMixer, NodeEvent, NoteEvent, NoteRange, SoundFontBuilder,
    SquareWaveSource,
};
use std::{thread::sleep, time::Duration};

fn main() {
    let square_font = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_inclusive_range(0, 50),
            Box::new(SquareWaveSource::new(None, 0.125, 0.5)),
        )
        .unwrap()
        .add_range(
            NoteRange::new_inclusive_range(51, 255),
            Box::new(SquareWaveSource::new(None, 0.125, 0.875)),
        )
        .unwrap()
        .build();
    let (mut sender, receiver) = AsyncEventReceiver::new(None, Box::new(square_font));
    let mixer = BaseMixer::from_consumer(Box::new(receiver));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::spawn(move || {
        sleep(Duration::from_millis(50));
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 69,
                event: NoteEvent::NoteOn { vel: 1.0 },
            },
        );
        sleep(Duration::from_millis(1500));
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 69,
                event: NoteEvent::NoteOff { vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 73,
                event: NoteEvent::NoteOn { vel: 0.375 },
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 73,
                event: NoteEvent::NoteOff { vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 74,
                event: NoteEvent::NoteOn { vel: 0.75 },
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 74,
                event: NoteEvent::NoteOff { vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 71,
                event: NoteEvent::NoteOn { vel: 0.375 },
            },
        );
        sleep(Duration::from_millis(500));
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 71,
                event: NoteEvent::NoteOff { vel: 0.0 },
            },
        );
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 69,
                event: NoteEvent::NoteOn { vel: 1.0 },
            },
        );
        sleep(Duration::from_millis(1000));
        send_or_log(
            &mut sender,
            &NodeEvent::Note {
                note: 69,
                event: NoteEvent::NoteOff { vel: 0.0 },
            },
        );
    });
    sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}

fn send_or_log(sender: &mut Sender<NodeEvent>, event: &NodeEvent) {
    if let Err(error) = sender.send(event.clone()) {
        println!("Send error: {:?}", error);
    }
}
