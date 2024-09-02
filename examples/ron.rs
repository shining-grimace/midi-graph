extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{util::config_from_file, BaseMixer, MidiSource};
use std::time::Duration;

const RON_FILE: &'static str = "resources/example.ron";

fn main() {
    let config = config_from_file(RON_FILE).unwrap();
    let midi = MidiSource::from_config(config).unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
