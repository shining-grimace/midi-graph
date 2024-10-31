use crate::{util::midi_builder_from_bytes, BaseMixer, MidiSource, SquareWaveSource};
use cpal::traits::StreamTrait;
use std::time::Duration;
use wasm_bindgen::prelude::*;

const MIDI_FILE: &'static [u8] = include_bytes!("../resources/dansenapolitaine.mid");

#[wasm_bindgen]
pub fn play_stream() {
    let midi_source = midi_builder_from_bytes(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(0, Box::new(SquareWaveSource::new(None, 0.25, 0.125)))
        .build()
        .unwrap();
    let mixer = BaseMixer::from_source(midi_source);
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
