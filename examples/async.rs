extern crate midi_graph;

use crossbeam_channel::Sender;
use midi_graph::{
    AsyncEventReceiver, BaseMixer, Fader, MixerSource, NodeControlEvent, NodeEvent, NoteEvent,
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
            &NodeEvent::Note {
                note: 69,
                event: NoteEvent::NoteOn { vel: 1.0 },
            },
        );
        for _ in 0..10 {
            sleep(Duration::from_millis(100));
            send_or_log(
                &mut sender,
                &NodeEvent::NodeControl {
                    node_id: MIXER_NODE_ID,
                    event: NodeControlEvent::MixerBalance(0.625),
                },
            );
            sleep(Duration::from_millis(100));
            send_or_log(
                &mut sender,
                &NodeEvent::NodeControl {
                    node_id: MIXER_NODE_ID,
                    event: NodeControlEvent::MixerBalance(0.375),
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
            &NodeEvent::NodeControl {
                node_id: FADER_NODE_ID,
                event: NodeControlEvent::Fade {
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
}

fn send_or_log(sender: &mut Sender<NodeEvent>, event: &NodeEvent) {
    if let Err(error) = sender.send(event.clone()) {
        println!("Send error: {:?}", error);
    }
}
