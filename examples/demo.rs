
extern crate midi_graph;

use midi_graph::{MidiProcessor, SquareAudio};
use std::time::Duration;
use cpal::traits::StreamTrait;

const MIDI_FILE: &'static str = "resources/MIDI_sample.mid";

fn main() {
    let smf = MidiProcessor::from_file(MIDI_FILE).unwrap();
    let streamer = SquareAudio::default();
    let stream = smf.open_stream(streamer).expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
