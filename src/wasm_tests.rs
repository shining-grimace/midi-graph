extern crate wasm_bindgen_test;

use crate::util::{midi_builder_from_bytes, wav_from_bytes};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const MIDI_FILE: &[u8] = include_bytes!("../resources/sample-in-c.mid");
const WAV_FILE: &[u8] = include_bytes!("../resources/guitar-a2-48k-stereo.wav");

#[wasm_bindgen_test]
fn pass() {
    // Test MIDI file
    let midi_builder = midi_builder_from_bytes(None, MIDI_FILE);
    assert!(midi_builder.is_ok());

    // Test wav file
    let wav = wav_from_bytes(WAV_FILE, 69, None, None);
    assert!(wav.is_ok());
}
