use crate::{AudioStreamer, Error};
use hound::{SampleFormat, WavSpec};

pub struct WavAudio {
    position: usize,
    length: usize,
    data: Vec<f32>,
}

impl WavAudio {
    pub fn new_from_data(spec: WavSpec, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        let length = data.len();
        Ok(Self {
            position: 0,
            length,
            data,
        })
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
}

impl AudioStreamer for WavAudio {
    fn is_completed(&self) -> bool {
        false
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let size = buffer.len();
        let samples_remaining = self.length - self.position;
        if samples_remaining < size {
            buffer.copy_from_slice(&self.data[self.position..(self.position + samples_remaining)]);
            &buffer[samples_remaining..size].fill(0.0);
        } else {
            buffer.copy_from_slice(&self.data[self.position..(self.position + size)]);
        }
    }
}
