extern crate midi_graph;

use cpal::traits::StreamTrait;
use crossbeam_channel::Sender;
use midi_graph::{
    AsyncEventReceiver, BaseMixer, ControlEvent, Fader, MixerSource, NodeEvent, NoteEvent,
    NoteRange, SawtoothWaveSource, SoundFontBuilder, SquareWaveSource, TriangleWaveSource,
};
use std::{thread::sleep, time::Duration};

const MIXER_NODE_ID: u64 = 100;
const FADER_NODE_ID: u64 = 101;

fn main() {
    let triangle_unison = MixerSource::new(
        Some(MIXER_NODE_ID),
        0.375,
        Box::new(TriangleWaveSource::new(None, 0.75)),
        Box::new(SawtoothWaveSource::new(None, 0.1625)),
    );

    let square_source = SquareWaveSource::new(None, 0.375, 0.25);
    let fader = Fader::new(Some(FADER_NODE_ID), 0.0, Box::new(square_source));
    let soundfont = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_inclusive_range(0, 70),
            Box::new(triangle_unison),
        )
        .unwrap()
        .add_range(NoteRange::new_inclusive_range(71, 255), Box::new(fader))
        .unwrap()
        .build();
    let (mut sender, receiver) = AsyncEventReceiver::new(None, Box::new(soundfont));
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
        println!("Sending fade event");
        send_or_log(
            &mut sender,
            &NodeEvent::Control {
                node_id: FADER_NODE_ID,
                event: ControlEvent::Fade {
                    from: 0.0,
                    to: 1.0,
                    seconds: 1.0,
                },
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
