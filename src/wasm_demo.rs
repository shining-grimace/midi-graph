use crate::{util::smf_from_bytes, BaseMixer, MidiSource, SquareWaveSource};
use cpal::traits::StreamTrait;
use std::time::Duration;
use wasm_bindgen::prelude::*;

const MIDI_FILE: &'static [u8] = include_bytes!("../resources/dansenapolitaine.mid");

#[wasm_bindgen]
pub fn play_stream() {
    let smf = smf_from_bytes(MIDI_FILE).unwrap();
    let midi = MidiSource::new(smf, || Box::new(SquareWaveSource::default()));
    let mixer = BaseMixer::from_source(midi);
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
