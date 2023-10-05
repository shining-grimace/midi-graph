
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
mod file;
mod mix;
mod source;

pub use error::Error;
pub use mix::midi::MidiProcessor;
pub use source::{
    AudioStreamer,
    square::SquareAudio
};

pub mod util {
    pub use crate::file::midi::{smf_from_file, smf_from_bytes};
    pub use crate::file::wav::{wav_from_file, wav_from_bytes};
}
