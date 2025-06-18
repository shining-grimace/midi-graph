extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, FileAssetLoader, GraphNode, NoteRange,
    generator::{LfsrNoiseNode, SquareWaveNode, TriangleWaveNode},
    group::FontNodeBuilder,
    util::midi_builder_from_file,
};
use std::time::Duration;

const MIDI_0_FILE: &'static str = "resources/sample-in-c.mid";
const MIDI_1_FILE: &'static str = "resources/LoopingMidi.mid";

const PROGRAM_0: usize = 0;
const PROGRAM_1: usize = 7;

fn main() {
    fn square_font() -> GraphNode {
        Box::new(
            FontNodeBuilder::new(None)
                .add_range(
                    NoteRange::new_full_range(),
                    Box::new(SquareWaveNode::new(None, Balance::Right, 0.125, 0.0625)),
                )
                .unwrap()
                .build(),
        )
    }
    fn triangle_font() -> GraphNode {
        Box::new(
            FontNodeBuilder::new(None)
                .add_range(
                    NoteRange::new_full_range(),
                    Box::new(TriangleWaveNode::new(None, Balance::Both, 1.0)),
                )
                .unwrap()
                .build(),
        )
    }
    fn noise_font() -> GraphNode {
        Box::new(
            FontNodeBuilder::new(None)
                .add_range(
                    NoteRange::new_full_range(),
                    Box::new(LfsrNoiseNode::new(None, Balance::Left, 0.25, false, 50)),
                )
                .unwrap()
                .build(),
        )
    }

    let program_0 = midi_builder_from_file(None, MIDI_0_FILE)
        .unwrap()
        .add_channel_source(0, triangle_font())
        .add_channel_source(1, square_font())
        .add_channel_source(2, noise_font())
        .build()
        .unwrap();
    let program_1 = midi_builder_from_file(None, MIDI_1_FILE)
        .unwrap()
        .add_channel_source(0, noise_font())
        .add_channel_source(1, square_font())
        .build()
        .unwrap();

    let mut mixer = BaseMixer::builder(FileAssetLoader, |_| {})
        .unwrap()
        .store_program(PROGRAM_0, Box::new(program_0))
        .store_program(PROGRAM_1, Box::new(program_1))
        .start(None)
        .unwrap();
    std::thread::sleep(Duration::from_secs(1));
    mixer.change_program(PROGRAM_0).unwrap();
    std::thread::sleep(Duration::from_secs(6));
    mixer.change_program(PROGRAM_1).unwrap();
    std::thread::sleep(Duration::from_secs(6));
}
