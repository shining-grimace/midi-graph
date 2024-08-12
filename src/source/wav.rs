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
    }

    fn on_note_off(&mut self, key: u8) {
        if self.current_note != key {
            return;
        }
        self.position = self.source_data.len();
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % config::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(config::CHANNEL_COUNT, 2);

        // Output properties
        let samples_can_write = buffer.len();
        let frames_can_write = samples_can_write / config::CHANNEL_COUNT;

        // Input properties
        let relative_pitch = util::relative_pitch_ratio_of(self.current_note) as f64;
        let source_samples_remaining = self.source_data.len() - self.position;
        let source_frames_remaining = source_samples_remaining / config::CHANNEL_COUNT;

        // Transfer alignment
        let source_frames_per_output_frame = 1.0 / relative_pitch;
        let expected_source_frames = {
            let unrounded = (frames_can_write as f64 * source_frames_per_output_frame) as usize;
            (unrounded / config::CHANNEL_COUNT) * config::CHANNEL_COUNT
        };
        let frames_will_transfer = expected_source_frames.min(source_frames_remaining);
        let frames_will_write = match frames_will_transfer < source_frames_remaining {
            true => {
                let unrounded =
                    (frames_will_transfer as f64 / source_frames_per_output_frame) as usize;
                (unrounded / config::CHANNEL_COUNT) * config::CHANNEL_COUNT
            }
            false => frames_can_write,
        };

        let source = &self.source_data
            [self.position..(self.position + frames_will_transfer * config::CHANNEL_COUNT)];
        for i in 0..frames_will_write {
            let output_index = i * config::CHANNEL_COUNT;
            let source_index =
                (i as f64 * source_frames_per_output_frame) as usize * config::CHANNEL_COUNT;
            buffer[output_index] += source[source_index];
            buffer[output_index + 1] += source[source_index + 1];
        }
        self.position = (self.position + (frames_will_write * config::CHANNEL_COUNT))
            .min(self.source_data.len());
    }
}
