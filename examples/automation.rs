extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer,
    generator::{SquareWaveNode, TriangleWaveNode},
    group::CombinerNode,
    util::midi_builder_from_file,
};
use std::time::Duration;

// MIDI file with two tracks, and these kinds of events:
// - Tempo change
// - Pan automation on the first track
// - Pitch automation on the first track
// - Volume automation on the second track
const MIDI_FILE: &'static str = "resources/MIDIGraphAutomationTest.mid";

const CHANNEL_NO: usize = 0;
const NODE_ID_BASS: u64 = 0;
const NODE_ID_LEAD: u64 = 1;
const TRACK_NO_BASS: usize = 1;
const TRACK_NO_LEAD: usize = 2;

fn main() {
    let bass_track_node = TriangleWaveNode::new(Some(NODE_ID_BASS), Balance::Both, 1.0);
    let bass_track_midi = midi_builder_from_file(None, MIDI_FILE, TRACK_NO_BASS)
        .unwrap()
        .add_channel_source(CHANNEL_NO, Box::new(bass_track_node))
        .build()
        .unwrap();
    let lead_track_node = SquareWaveNode::new(Some(NODE_ID_LEAD), Balance::Both, 0.25, 0.0625);
    let lead_track_midi = midi_builder_from_file(None, MIDI_FILE, TRACK_NO_LEAD)
        .unwrap()
        .add_channel_source(CHANNEL_NO, Box::new(lead_track_node))
        .build()
        .unwrap();
    let combiner_node = CombinerNode::new(
        None,
        vec![Box::new(bass_track_midi), Box::new(lead_track_midi)],
    );
    let _mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(combiner_node))
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
