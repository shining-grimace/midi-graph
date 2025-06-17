extern crate midi_graph;

use midi_graph::{BaseMixer, FileAssetLoader, abstraction::NodeConfigData, group::Subtree};
use std::time::Duration;

const JSON_FILE: &'static str = "resources/json-example.json";

fn main() {
    let subtree = Subtree::as_path(JSON_FILE);
    let _mixer = BaseMixer::builder(FileAssetLoader::default(), |_| {})
        .unwrap()
        .set_initial_program_from_config(1, NodeConfigData(Box::new(subtree)))
        .unwrap()
        .build(1)
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
