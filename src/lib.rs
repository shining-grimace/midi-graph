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
mod event;
mod file;
mod loader;
mod mix;
mod node;

// Re-exports as new types
pub type MessageSender = crossbeam_channel::Sender<Message>;

// Helper types
pub type GraphNode = Box<dyn Node + Send + 'static>;

// General exports below

pub use config::{Config, FontSource, Loop, MidiDataSource, RangeSource, SoundSource};
pub use error::Error;
pub use event::{Balance, Event, EventTarget, Message};
pub use file::loader::FileGraphLoader;
pub use loader::GraphLoader;
pub use mix::base::BaseMixer;
pub use node::{LoopRange, Node, NoteRange};

pub mod effect {
    pub use crate::node::effect::{
        ModulationProperty,
        adsr::AdsrEnvelope,
        fader::Fader,
        lfo::Lfo,
        transition::TransitionEnvelope,
    };
}

pub mod generator {
    pub use crate::node::generator::{
        noise::LfsrNoiseSource, null::NullSource, one_shot::OneShotSource,
        sawtooth::SawtoothWaveSource, square::SquareWaveSource, triangle::TriangleWaveSource,
        wav::WavSource,
    };
}

pub mod group {
    pub use crate::node::group::{
        combiner::CombinerSource,
        mixer::MixerSource,
        polyphony::Polyphony,
        font::{SoundFont, SoundFontBuilder}
    };
}

pub mod midi {
    pub use crate::node::midi::{
        MidiSource, MidiSourceBuilder, cue::CueData, util::MidiEvent,
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
