extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{util::smf_from_file, util::wav_from_file, BaseMixer, MidiSource};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/MIDI_sample.mid";
const WAV_FILE: &'static str = "resources/kick.wav";

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let wav = wav_from_file(WAV_FILE).unwrap();
    let midi = MidiSource::new(smf, Box::new(wav)).unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
