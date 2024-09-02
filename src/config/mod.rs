use serde_derive::Deserialize;
use std::collections::HashMap;

pub type DutyCycle = f32;
pub type Amplitude = f32;
pub type BaseNote = u8;
pub type InstrumentIndex = usize;

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
    Sf2FilePath(String, InstrumentIndex),
}

#[derive(Deserialize)]
pub struct RangeSource {
    pub source: SoundSource,
    pub lower: u8,
    pub upper: u8,
}

#[derive(Deserialize)]
pub enum SoundSource {
    SquareWave(Amplitude, DutyCycle),
    TriangleWave(Amplitude),
    SampleFilePath(String, BaseNote),
}
