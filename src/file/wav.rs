use crate::Error;
use hound::{Sample, SampleFormat, WavReader, WavSpec};
use std::io::Cursor;

pub fn wav_from_file(file_name: &str) -> Result<(WavSpec, Vec<f32>), Error> {
    let wav = WavReader::open(file_name)?;
    let spec = wav.spec();
    validate_spec(&spec)?;
    let samples = wav.into_samples().map(|s| s.unwrap()).collect();
    Ok((spec, samples))
}

pub fn wav_from_bytes<S: Sample>(bytes: &'static [u8]) -> Result<(WavSpec, Vec<f32>), Error> {
    let cursor = Cursor::new(bytes);
    let wav = WavReader::new(cursor)?;
    let spec = wav.spec();
    validate_spec(&spec)?;
    let samples = wav.into_samples().map(|s| s.unwrap()).collect();
    Ok((spec, samples))
}

fn validate_spec(spec: &WavSpec) -> Result<(), Error> {
    if spec.channels != 1 {
        return Err(Error::User(format!(
            "{} channels is not supported",
            spec.channels
        )));
    }
    if spec.sample_rate != 48000 {
        return Err(Error::User(format!(
            "{} samples per second is not supported",
            spec.sample_rate
        )));
    }
    if spec.sample_format != SampleFormat::Float {
        return Err(Error::User(format!(
            "Sample format {:?} is not supported",
            spec.sample_format
        )));
    }
    if spec.bits_per_sample != 32 {
        return Err(Error::User(format!(
            "{} bits per sample is not supported",
            spec.bits_per_sample
        )));
    }
    Ok(())
}
