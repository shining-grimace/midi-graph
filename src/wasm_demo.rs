use crate::{Balance, BaseMixer, generator::SquareWaveNode, util::midi_builder_from_bytes};
use std::time::Duration;
use wasm_bindgen::prelude::*;

const MIDI_FILE: &'static [u8] = include_bytes!("../resources/LoopingMidi.mid");

#[wasm_bindgen]
pub fn play_stream() {
    let midi_source = midi_builder_from_bytes(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(
            0,
            Box::new(SquareWaveNode::new(None, Balance::Both, 0.25, 0.125)),
        )
        .build()
        .unwrap();
    let _mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(midi_source))
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(5));
}
