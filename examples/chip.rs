extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, FileAssetLoader, NoteRange,
    generator::{LfsrNoiseNode, SawtoothWaveNode, SquareWaveNode, TriangleWaveNode},
    group::{FontNodeBuilder, MixerNode, PolyphonyNode},
    util::midi_builder_from_file,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const TRIANGLE_CHANNEL: usize = 0;
const SQUARE_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

fn main() {
    let triangle_unison = PolyphonyNode::new(
        None,
        4,
        Box::new(MixerNode::new(
            None,
            0.5,
            Box::new(TriangleWaveNode::new(None, Balance::Left, 1.0)),
            Box::new(SawtoothWaveNode::new(None, Balance::Right, 0.25)),
        )),
    )
    .unwrap();
    let triangle_font = FontNodeBuilder::new(None)
        .add_range(NoteRange::new_full_range(), Box::new(triangle_unison))
        .unwrap()
        .build();
    let square_font = FontNodeBuilder::new(None)
        .add_range(
            NoteRange::new_inclusive_range(0, 50),
            Box::new(SquareWaveNode::new(None, Balance::Both, 0.125, 0.5)),
        )
        .unwrap()
        .add_range(
            NoteRange::new_inclusive_range(51, 255),
            Box::new(SquareWaveNode::new(None, Balance::Both, 0.125, 0.875)),
        )
        .unwrap()
        .build();
    let noise_font = FontNodeBuilder::new(None)
        .add_range(
            NoteRange::new_full_range(),
            Box::new(LfsrNoiseNode::new(None, Balance::Both, 0.25, false, 50)),
        )
        .unwrap()
        .build();
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(TRIANGLE_CHANNEL, Box::new(triangle_font))
        .add_channel_source(SQUARE_CHANNEL, Box::new(square_font))
        .add_channel_source(NOISE_CHANNEL, Box::new(noise_font))
        .build()
        .unwrap();
    let _mixer = BaseMixer::builder(FileAssetLoader, |_| {})
        .unwrap()
        .set_initial_program(1, Box::new(midi))
        .build(1)
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
