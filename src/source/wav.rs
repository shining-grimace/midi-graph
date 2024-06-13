use crate::{config, util, AudioSource, Error};
use hound::{SampleFormat, WavSpec};

pub struct WavSource {
    position: usize,
    current_note: u8,
    source_data: Vec<f32>,
}

impl WavSource {
    pub fn new_from_data(spec: WavSpec, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        Ok(Self {
            position: 0,
            current_note: 0,
            source_data: data,
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

impl AudioSource for WavSource {
    fn on_note_on(&mut self, key: u8) {
        self.position = 0;
        self.current_note = key;

        let relative_pitch = util::relative_pitch_of(key) as f64;
    }

    fn on_note_off(&mut self, key: u8) {
        if self.current_note != key {
            return;
        }
        self.position = self.source_data.len();
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let size = buffer.len();

        #[cfg(debug_assertions)]
        assert_eq!(size % config::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(config::CHANNEL_COUNT, 2);

        let samples_can_play = size / config::CHANNEL_COUNT;
        let samples_remaining = self.source_data.len() - self.position;
        let samples_will_play = samples_can_play.min(samples_remaining);
        let buffer_length_to_write = samples_will_play * config::CHANNEL_COUNT;
        let source = &self.source_data[self.position..(self.position + samples_will_play)];
        let mut source_index = 0;
        for i in (0..buffer_length_to_write).step_by(config::CHANNEL_COUNT) {
            let sample = source[source_index];
            buffer[i] += sample;
            buffer[i + 1] += sample;
            source_index += 1;
        }
        self.position = (self.position + samples_will_play).min(self.source_data.len());
    }
}
