extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, FileAssetLoader,
    abstraction::{ChildConfig, NodeConfig},
    generator::{SquareWave, TriangleWave},
    group::CombinerNode,
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

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
    let mut asset_loader = FileAssetLoader::default();
    let bass_track_instrument = TriangleWave {
        node_id: Some(NODE_ID_BASS),
        balance: Balance::Both,
        amplitude: 1.0,
    };
    let bass_track_midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: TRACK_NO_BASS,
        },
        channels: HashMap::from([(CHANNEL_NO, ChildConfig(Box::new(bass_track_instrument)))]),
    };
    let bass_track_midi_node = bass_track_midi.to_node(&mut asset_loader).unwrap();
    let lead_track_instrument = SquareWave {
        node_id: Some(NODE_ID_LEAD),
        balance: Balance::Both,
        amplitude: 0.25,
        duty_cycle: 0.0625,
    };
    let lead_track_midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: TRACK_NO_LEAD,
        },
        channels: HashMap::from([(CHANNEL_NO, ChildConfig(Box::new(lead_track_instrument)))]),
    };
    let lead_track_midi_node = lead_track_midi.to_node(&mut asset_loader).unwrap();
    let combiner_node = CombinerNode::new(None, vec![bass_track_midi_node, lead_track_midi_node]);
    let _mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(combiner_node))
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
