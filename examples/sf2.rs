extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{util::smf_from_file, util::soundfont_from_file, BaseMixer, MidiSource};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const SF2_FILE: &'static str = "resources/german8-harpsichord.sf2";

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let fonts = (0..smf.tracks.len())
        .map(|_| soundfont_from_file(SF2_FILE, 0).unwrap())
        .collect();
    let midi = MidiSource::new(smf, fonts).unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
