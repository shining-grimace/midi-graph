use crate::{util::midi_builder_from_bytes, BaseMixer, generator::SquareWaveSource};
use std::time::Duration;
use wasm_bindgen::prelude::*;

const MIDI_FILE: &'static [u8] = include_bytes!("../resources/LoopingMidi.mid");

#[wasm_bindgen]
pub fn play_stream() {
    let midi_source = midi_builder_from_bytes(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(0, Box::new(SquareWaveSource::new(None, 0.25, 0.125)))
        .build()
        .unwrap();
    let _mixer = BaseMixer::start_single_program(Box::new(midi_source));
    std::thread::sleep(Duration::from_secs(5));
}
