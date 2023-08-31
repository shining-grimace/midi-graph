

extern crate wasm_bindgen_test;

use crate::MidiProcessor;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const TEST_FILE: &[u8] = include_bytes!("../resources/MIDI_sample.mid");

#[wasm_bindgen_test]
fn pass() {
    let smf = MidiProcessor::from_bytes(TEST_FILE);
    assert!(smf.is_ok());
}
