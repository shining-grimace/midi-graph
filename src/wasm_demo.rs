
use crate::{MidiProcessor, SquareAudio, util::smf_from_bytes};
use std::time::Duration;
use wasm_bindgen::prelude::*;
use cpal::traits::StreamTrait;

const MIDI_FILE: &'static [u8] = include_bytes!("../resources/MIDI_sample.mid");

#[wasm_bindgen]
pub fn play_stream() {
    let smf = smf_from_bytes(MIDI_FILE).unwrap();
    let midi = MidiProcessor::from_file(smf);
    let streamer = SquareAudio::default();
    let stream = midi.open_stream(streamer).expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
