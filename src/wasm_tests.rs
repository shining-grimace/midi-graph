
extern crate wasm_bindgen_test;

use crate::util::{smf_from_bytes, wav_from_bytes};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const MIDI_FILE: &[u8] = include_bytes!("../resources/MIDI_sample.mid");
const WAV_FILE: &[u8] = include_bytes!("../resources/snare.wav");

#[wasm_bindgen_test]
fn pass() {

    // Test MIDI file
    let smf = smf_from_bytes(MIDI_FILE);
    assert!(smf.is_ok());

    // Test wav file
    let wav = wav_from_bytes(WAV_FILE);
    assert!(wav.is_ok());
}
