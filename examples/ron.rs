extern crate midi_graph;

use midi_graph::{
    util::{config_from_file, source_from_config},
    BaseMixer,
};
use std::time::Duration;

const RON_FILE: &'static str = "resources/example.ron";

fn main() {
    let config = config_from_file(RON_FILE).unwrap();
    let source = source_from_config(&config.root).unwrap();
    let _mixer = BaseMixer::start_with(source).expect("Could not open stream");
    std::thread::sleep(Duration::from_secs(16));
}
