use crate::{Error, WavSource};
use hound::WavReader;
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
