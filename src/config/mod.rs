use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Config {
    pub midi: MidiDataSource,
    pub channels: HashMap<usize, FontSource>,
}

#[derive(Deserialize)]
pub enum MidiDataSource {
    FilePath(String),
}

#[derive(Deserialize)]
pub enum FontSource {
    Ranges(Vec<RangeSource>),
    Sf2FilePath {
        path: String,
        instrument_index: usize,
    },
}

#[derive(Deserialize)]
pub struct RangeSource {
    pub source: SoundSource,
    pub lower: u8,
    pub upper: u8,
}

#[derive(Deserialize)]
pub enum SoundSource {
    SquareWave {
        amplitude: f32,
        duty_cycle: f32,
    },
    TriangleWave {
        amplitude: f32,
    },
    LfsrNoise {
        amplitude: f32,
        inside_feedback: bool,
        note_for_16_shifts: u8,
    },
    SampleFilePath {
        path: String,
        base_note: u8,
    },
}
