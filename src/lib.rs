
// Test suite for the Web and headless browsers.
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_tests;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
mod wasm_demo;

mod error;
mod source;

pub use error::Error;
pub use source::{
    AudioStreamer,
    midi::MidiProcessor,
    square::SquareAudio
};
