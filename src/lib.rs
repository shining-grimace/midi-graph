// Test suite for the Web and headless browsers.
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_tests;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
mod wasm_demo;

mod config;
mod error;
mod file;
mod loader;
mod mix;
mod node;

pub use config::{Config, FontSource, Loop, MidiDataSource, RangeSource, SoundSource};
pub use error::Error;
pub use file::loader::FileGraphLoader;
pub use loader::GraphLoader;
pub use mix::base::BaseMixer;
pub use node::{
    BroadcastControl, LoopRange, Node, NodeControlEvent, NodeEvent, NoteEvent, NoteRange,
};

pub mod effect {
    pub use crate::node::effect::{
        async_receiver::{AsyncEventReceiver, EventChannel},
        envelope::Envelope,
        fader::Fader
    };
}

pub mod font {
    pub use crate::node::font::{SoundFont, SoundFontBuilder};
}

pub mod generator {
    pub use crate::node::generator::{
        noise::LfsrNoiseSource,
        null::NullSource,
        one_shot::OneShotSource,
        sawtooth::SawtoothWaveSource,
        square::SquareWaveSource,
        triangle::TriangleWaveSource,
        wav::WavSource
    };
}

pub mod group {
    pub use crate::node::group::{
        combiner::CombinerSource,
        mixer::MixerSource,
        polyphony::Polyphony
    };
}

pub mod midi {
    pub use crate::node::midi::{
        cue::{Cue, TimelineCue},
        MidiSource, MidiSourceBuilder,
    };
}

pub mod util {
    pub use crate::file::font::*;
    pub use crate::file::midi::*;
    pub use crate::file::wav::*;
    pub use crate::node::midi::util::*;
    pub use crate::node::util::*;
}

pub mod consts {
    pub const PLAYBACK_SAMPLE_RATE: usize = 48000;
    pub const CHANNEL_COUNT: usize = 2;
    pub const BUFFER_SIZE: usize = 2048;
}
