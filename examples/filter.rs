extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, IirFilter, Message, NoteRange,
    effect::FilterNode,
    generator::{LfsrNoiseNode, SawtoothWaveNode, SquareWaveNode, TriangleWaveNode},
    group::{FontNodeBuilder, MixerNode, PolyphonyNode},
    util::midi_builder_from_file,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const TRIANGLE_CHANNEL: usize = 0;
const SQUARE_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

const FILTER_NODE_ID: u64 = 1;

fn main() {
    let triangle_unison = PolyphonyNode::new(
        None,
        4,
        Box::new(MixerNode::new(
            None,
            0.5,
            Box::new(TriangleWaveNode::new(None, Balance::Left, 0.75)),
            Box::new(SawtoothWaveNode::new(None, Balance::Right, 0.1875)),
        )),
    )
    .unwrap();
    let triangle_font = FontNodeBuilder::new(None)
        .add_range(NoteRange::new_full_range(), Box::new(triangle_unison))
        .unwrap()
        .build();
    let square_node = FilterNode::new(
        Some(FILTER_NODE_ID),
        Some((IirFilter::LowPass, 1000.0)),
        Box::new(
            PolyphonyNode::new(
                None,
                4,
                Box::new(SquareWaveNode::new(None, Balance::Both, 0.125, 0.5)),
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let noise_font = FontNodeBuilder::new(None)
        .add_range(
            NoteRange::new_full_range(),
            Box::new(LfsrNoiseNode::new(None, Balance::Both, 0.25, false, 50)),
        )
        .unwrap()
        .build();
    let midi = midi_builder_from_file(None, MIDI_FILE, 0)
        .unwrap()
        .add_channel_source(TRIANGLE_CHANNEL, Box::new(triangle_font))
        .add_channel_source(SQUARE_CHANNEL, Box::new(square_node))
        .add_channel_source(NOISE_CHANNEL, Box::new(noise_font))
        .build()
        .unwrap();
    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(midi))
        .start(Some(1))
        .unwrap();
    let sender = mixer.get_event_sender();
    std::thread::sleep(Duration::from_secs(8));
    sender
        .send(Message {
            target: EventTarget::SpecificNode(FILTER_NODE_ID),
            data: Event::Filter {
                filter: IirFilter::HighPass,
                cutoff_frequency: 4000.0,
            },
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(4));
    sender
        .send(Message {
            target: EventTarget::SpecificNode(FILTER_NODE_ID),
            data: Event::FilterFrequencyShift(-3000.0),
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(4));
}
