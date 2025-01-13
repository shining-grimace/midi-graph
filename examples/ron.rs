extern crate midi_graph;

use midi_graph::{BaseMixer, FileGraphLoader};
use std::time::Duration;

const RON_FILE: &'static str = "resources/example.ron";

fn main() {
    let loader = FileGraphLoader::default();
    let config = loader.config_from_file(RON_FILE).unwrap();
    let _mixer = BaseMixer::start_single_program_from_config(&loader, None, &config)
        .expect("Could not open stream");
    std::thread::sleep(Duration::from_secs(16));
}
