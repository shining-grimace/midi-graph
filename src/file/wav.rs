use crate::{Error, WavSource};
use hound::WavReader;
use soundfont::data::SampleHeader;

use std::io::Cursor;

/// Make a WavSource. The source note is a MIDI notes, where 69 is A440.
pub fn wav_from_file(file_name: &str, source_note: u8) -> Result<WavSource, Error> {
    let wav = WavReader::open(file_name)?;
    let spec = wav.spec();
    let samples = wav.into_samples().map(|s| s.unwrap()).collect();
    WavSource::new_from_data(spec, source_note, samples)
}

/// Make a WavSource. The source note is a MIDI notes, where 69 is A440.
pub fn wav_from_bytes(bytes: &'static [u8], source_note: u8) -> Result<WavSource, Error> {
    let cursor = Cursor::new(bytes);
    let wav = WavReader::new(cursor)?;
    let spec = wav.spec();
    let samples = wav.into_samples().map(|s| s.unwrap()).collect();
    WavSource::new_from_data(spec, source_note, samples)
}

pub fn wav_from_i16_samples(header: &SampleHeader, data: &Vec<i16>) -> Result<WavSource, Error> {
    let mut float_data = vec![0.0; data.len()];
    for (i, sample) in data.iter().enumerate() {
        float_data[i] = *sample as f32 / 32768.0;
    }
    WavSource::new_from_raw_data(header, float_data)
}
