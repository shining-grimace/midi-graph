use crate::{Balance, Error};
use ron::de::from_bytes;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

const fn none_id() -> Option<u64> {
    None
}

const fn default_soundfont_polyphony_voices() -> usize {
    1
}

const fn default_amplitude() -> f32 {
    0.5
}

const fn default_duty_cycle() -> f32 {
    0.5
}

const fn default_note_for_16_shifts() -> u8 {
    64
}

const fn default_attack() -> f32 {
    0.125
}

const fn default_decay() -> f32 {
    0.25
}

const fn default_sustain() -> f32 {
    0.5
}

const fn default_release() -> f32 {
    0.125
}

const fn default_source_balance() -> Balance {
    Balance::Both
}

const fn default_mixer_balance() -> f32 {
    0.5
}

const fn default_max_voices() -> usize {
    4
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub root: SoundSource,
}

impl Config {
    pub fn from_bytes(bytes: &[u8]) -> Result<Config, Error> {
        let config = from_bytes(bytes)?;
        Ok(config)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub enum MidiDataSource {
    FilePath(String),
}

#[derive(Deserialize, Serialize, Clone)]
pub enum FontSource {
    Ranges(Vec<RangeSource>),
    Sf2FilePath {
        path: String,
        instrument_index: usize,
        #[serde(default = "default_soundfont_polyphony_voices")]
        polyphony_voices: usize
    },
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RangeSource {
    pub source: SoundSource,
    pub lower: u8,
    pub upper: u8,
}

/// Loop range, defined as the inclusive start and exclusive end.
/// These points are specified in frames, not data points.
#[derive(Deserialize, Serialize, Clone)]
pub struct Loop {
    pub start: usize,
    pub end: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum SoundSource {
    Midi {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        source: MidiDataSource,
        channels: HashMap<usize, SoundSource>,
    },
    EventReceiver {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        source: Box<SoundSource>,
    },
    Font {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        config: FontSource,
    },
    SquareWave {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_source_balance")]
        balance: Balance,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default = "default_duty_cycle")]
        duty_cycle: f32,
    },
    TriangleWave {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_source_balance")]
        balance: Balance,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
    },
    SawtoothWave {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_source_balance")]
        balance: Balance,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
    },
    LfsrNoise {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_source_balance")]
        balance: Balance,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        inside_feedback: bool,
        #[serde(default = "default_note_for_16_shifts")]
        note_for_16_shifts: u8,
    },
    SampleFilePath {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_source_balance")]
        balance: Balance,
        path: String,
        base_note: u8,
        looping: Option<Loop>,
    },
    OneShotFilePath {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_source_balance")]
        balance: Balance,
        path: String,
    },
    Envelope {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_attack")]
        attack_time: f32,
        #[serde(default = "default_decay")]
        decay_time: f32,
        #[serde(default = "default_sustain")]
        sustain_multiplier: f32,
        #[serde(default = "default_release")]
        release_time: f32,
        source: Box<SoundSource>,
    },
    Combiner {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        sources: Vec<SoundSource>,
    },
    Mixer {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_mixer_balance")]
        balance: f32,
        source_0: Box<SoundSource>,
        source_1: Box<SoundSource>,
    },
    Polyphony {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        #[serde(default = "default_max_voices")]
        max_voices: usize,
        source: Box<SoundSource>,
    },
    Fader {
        #[serde(default = "none_id")]
        node_id: Option<u64>,
        initial_volume: f32,
        source: Box<SoundSource>,
    },
}

impl SoundSource {
    pub const fn stock_square_wave() -> Self {
        SoundSource::SquareWave {
            node_id: none_id(),
            balance: Balance::Both,
            amplitude: default_amplitude(),
            duty_cycle: default_duty_cycle(),
        }
    }

    pub const fn stock_triangle_wave() -> Self {
        SoundSource::TriangleWave {
            node_id: none_id(),
            balance: Balance::Both,
            amplitude: default_amplitude(),
        }
    }

    pub const fn stock_sawtooth_wave() -> Self {
        SoundSource::SawtoothWave {
            node_id: none_id(),
            balance: Balance::Both,
            amplitude: default_amplitude(),
        }
    }

    pub fn event_receiver(node_id: Option<u64>, source: SoundSource) -> Self {
        SoundSource::EventReceiver {
            node_id,
            source: Box::new(source),
        }
    }

    pub fn stock_noise_source(inside_feedback_mode: bool) -> Self {
        SoundSource::LfsrNoise {
            node_id: none_id(),
            balance: Balance::Both,
            amplitude: default_amplitude(),
            inside_feedback: inside_feedback_mode,
            note_for_16_shifts: default_note_for_16_shifts(),
        }
    }

    pub fn stock_envelope(inner: SoundSource) -> Self {
        SoundSource::Envelope {
            node_id: none_id(),
            attack_time: default_attack(),
            decay_time: default_decay(),
            sustain_multiplier: default_sustain(),
            release_time: default_release(),
            source: Box::new(inner),
        }
    }

    pub fn stock_mixer(inner_0: SoundSource, inner_1: SoundSource) -> Self {
        SoundSource::Mixer {
            node_id: none_id(),
            balance: default_mixer_balance(),
            source_0: Box::new(inner_0),
            source_1: Box::new(inner_1),
        }
    }

    pub fn stock_polyphony(inner: SoundSource) -> Self {
        SoundSource::Polyphony {
            node_id: none_id(),
            max_voices: default_max_voices(),
            source: Box::new(inner)
        }
    }

    pub fn stock_fader(inner: SoundSource) -> Self {
        SoundSource::Fader {
            node_id: none_id(),
            initial_volume: 1.0,
            source: Box::new(inner),
        }
    }

    pub fn stock_full_range_font(source: SoundSource) -> Self {
        SoundSource::Font {
            node_id: none_id(),
            config: FontSource::Ranges(vec![RangeSource {
                source,
                lower: 0,
                upper: 127,
            }]),
        }
    }
}
