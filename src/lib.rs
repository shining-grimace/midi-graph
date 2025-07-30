// AND THEN DO AN INTO BOX

//! # MIDI Graph
//!
//! Cross-platform audio engine crafted for games.
//!
//! To start using this library, you'll want to build a graph of nodes to
//! shape the sound, and create a sequence of events to push notes and
//! modulations down the graph.
//!
//! ## Nodes
//!
//! Tones and layers are build by assembling nodes in a graph.
//!
//! See:
//! - [GraphNode] for the trait object type which all nodes implement
//! - [effect] for nodes that wrap another while applying effects
//! - [generator] for nodes with no children that produce sound
//! - [group] for nodes that wrap other nodes without applying effects
//! - [midi] for the special node that emits events over time
//!
//! ## Graph Representation
//!
//! Thw [Config] type is an abstract representation of a node graph. This can be
//! deserialised from a `.ron` file, or constructed in code. The
//!
//! ## Events
//!
//! Music is formed by sending events down the graph.
//!
//! See:
//! - [midi::MidiSource] for an emitter of events, prepared ahead of
//!   time from a MIDI file source or a custom list
//! - [MessageSender] for an asynchronous channel sender to queue events
//!   any time
//!
//! ## File Formats
//!
//! Utilities are provided to load files. See the [util] module.
//!
//! Supported file types are:
//! - MIDI files - `.mid` or `.smf`
//! - WAV audio files - `.wav`
//! - SoundFont 2 files - `.sf2`
//! - RON files - `.ron` (for loading graph configuration)

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
mod log;
mod mix;
mod node;

// Re-exports as new types
pub type MessageSender = crossbeam_channel::Sender<Message>;

// Helper types
pub type GraphNode = Box<dyn Node + Send + 'static>;

// General exports below

pub use error::Error;
pub use event::{Balance, Event, EventTarget, IirFilter, Message};
pub use file::FileAssetLoader;
pub use loader::{AssetLoadPayload, AssetLoader, SampleBuffer, SerializedFileMetadata};
pub use log::DebugLogging;
pub use mix::base::BaseMixer;
pub use node::{LoopRange, Node, NoteRange};

/// Abstract, serialisable/deserialisable representation of a graph
pub mod abstraction {
    pub mod defaults {
        pub use crate::config::defaults::*;
    }
    pub use super::config::{Loop, NodeConfig, NodeConfigData, registry::NodeRegistry};
}

/// Nodes that wrap other nodes and apply effects to them.
pub mod effect {
    pub use crate::node::effect::{
        ModulationProperty,
        adsr::{AdsrEnvelope, AdsrEnvelopeNode},
        fader::{Fader, FaderNode},
        filter::{Filter, FilterNode},
        lfo::{Lfo, LfoNode},
        transition::{Transition, TransitionNode},
    };
}

/// Nodes that create audio. These do not have child nodes; that is, they are leaf nodes.
pub mod generator {
    pub use crate::node::generator::{
        noise::{LfsrNoise, LfsrNoiseNode},
        null::{Null, NullNode},
        one_shot::{OneShot, OneShotFileMetadata, OneShotNode},
        sawtooth::{SawtoothWave, SawtoothWaveNode},
        square::{SquareWave, SquareWaveNode},
        triangle::{TriangleWave, TriangleWaveNode},
        wav::{SampleLoop, SampleLoopNode},
    };
}

/// Nodes that wrap and orchestrate child nodes
pub mod group {
    pub use crate::node::group::{
        combiner::{Combiner, CombinerNode},
        font::{Font, FontNode, FontNodeBuilder, FontSource, RangeSource},
        mixer::{Mixer, MixerNode},
        polyphony::{Polyphony, PolyphonyNode},
        subtree::{Subtree, SubtreeData},
    };
}

/// Special node that plays through a pre-defined, timed event sequence
pub mod midi {
    pub use crate::node::midi::{
        Midi, MidiDataSource, MidiNode, MidiNodeBuilder, cue::CueData, event::MidiEvent,
    };
}

/// Utilities for opening files and doing frequency calculations
pub mod util {
    pub use crate::node::midi::util::*;
    pub use crate::node::util::*;
}

/// Locked properties of the audio output stream
pub mod consts {
    pub const PLAYBACK_SAMPLE_RATE: usize = 48000;
    pub const CHANNEL_COUNT: usize = 2;
    pub const BUFFER_SIZE: usize = 2048;
}
