extern crate midi_graph;

use midi_graph::{
    AssetLoader, BaseMixer, FileAssetLoader, abstraction::NodeConfigData, group::Subtree,
};
use std::time::Duration;

const JSON_FILE: &'static str = "resources/json-example.json";

fn main() {
    let subtree = Subtree::as_path(JSON_FILE);
    let asset_loader: Box<dyn AssetLoader> = Box::new(FileAssetLoader);
    let _mixer = BaseMixer::builder(|_| {})
        .unwrap()
        .set_initial_program_from_config(1, NodeConfigData(Box::new(subtree)), &asset_loader)
        .unwrap()
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
