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
pub use mix::base::BaseMixer;
pub use source::{
    midi::{chunk::MidiChunkSource, track::MidiTrackSource, MidiSource},
    square::SquareWaveSource,
    wav::WavSource,
    AudioSource,
};

pub mod util {
    pub use crate::file::midi::*;
    pub use crate::file::wav::*;
    pub use crate::source::midi::util::*;
    pub use crate::source::util::*;
}

pub mod constants {
    pub const PLAYBACK_SAMPLE_RATE: usize = 48000;
}
