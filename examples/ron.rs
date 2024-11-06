extern crate midi_graph;

use midi_graph::{util::config_from_file, BaseMixer, MidiSource};
use std::time::Duration;

const RON_FILE: &'static str = "resources/example.ron";

fn main() {
    let config = config_from_file(RON_FILE).unwrap();
    let midi = MidiSource::from_config(config).unwrap();
    let mixer = BaseMixer::start_with(Box::new(midi)).expect("Could not open stream");
    std::thread::sleep(Duration::from_secs(16));
}
