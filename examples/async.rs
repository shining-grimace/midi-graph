extern crate midi_graph;

use cpal::traits::StreamTrait;
use crossbeam_channel::Sender;
use midi_graph::{
    AsyncEventReceiver, BaseMixer, ControlEvent, MixerSource, NodeEvent, NoteEvent, NoteRange,
    SawtoothWaveSource, SoundFontBuilder, SquareWaveSource, TriangleWaveSource,
};
use std::{thread::sleep, time::Duration};

const MIXER_NODE_ID: u64 = 100;

fn main() {
    let triangle_unison = MixerSource::new(
        Some(MIXER_NODE_ID),
        0.375,
        Box::new(TriangleWaveSource::new(None, 0.75)),
        Box::new(SawtoothWaveSource::new(None, 0.1625)),
    );
    let square_font = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_inclusive_range(0, 70),
            Box::new(triangle_unison),
        )
        .unwrap()
        .add_range(
            NoteRange::new_inclusive_range(71, 255),
            Box::new(SquareWaveSource::new(None, 0.25, 0.875)),
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
        for _ in 0..10 {
            sleep(Duration::from_millis(100));
            send_or_log(
                &mut sender,
                &NodeEvent::Control {
                    node_id: MIXER_NODE_ID,
                    event: ControlEvent::MixerBalance(0.625),
                },
            );
            sleep(Duration::from_millis(100));
            send_or_log(
                &mut sender,
                &NodeEvent::Control {
                    node_id: MIXER_NODE_ID,
                    event: ControlEvent::MixerBalance(0.375),
                },
            );
        }
        sleep(Duration::from_millis(500));
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
