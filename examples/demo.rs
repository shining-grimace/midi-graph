extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{util::smf_from_file, BaseMixer, SquareAudio};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/MIDI_sample.mid";

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let smf = BaseMixer::from_file(smf);
    let streamer = SquareAudio::default();
    let stream = smf.open_stream(streamer).expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
