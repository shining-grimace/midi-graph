use crate::{config, util, BufferConsumer, Error, NoteEvent};
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

impl BufferConsumer for WavSource {
    fn set_note(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::NoteOn(note) => {
                self.position = 0;
                self.current_note = note;
            }
            NoteEvent::NoteOff(note) => {
                if self.current_note != note {
                    return;
                }
                self.position = self.source_data.len();
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % config::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(config::CHANNEL_COUNT, 2);

        // Scaling
        let relative_pitch = util::relative_pitch_ratio_of(self.current_note) as f64;
        let source_frames_per_output_frame = 1.0 / relative_pitch;

        // Output properties
        let samples_can_write = buffer.len();
        let frames_can_write = samples_can_write / config::CHANNEL_COUNT;

        // Input properties
        let source_samples_remaining = self.source_data.len() - self.position;
        let source_frames_remaining = source_samples_remaining / config::CHANNEL_COUNT;

        // Transfer alignment
        let needed_source_frames = {
            let unrounded = (frames_can_write as f64 * source_frames_per_output_frame) as usize;
            unrounded - (unrounded % config::CHANNEL_COUNT)
        };
        let enough_frames_in_source = needed_source_frames <= source_frames_remaining;
        let frames_will_write = match enough_frames_in_source {
            true => frames_can_write,
            false => {
                let unrounded =
                    (source_frames_remaining as f64 / source_frames_per_output_frame) as usize;
                unrounded - unrounded % config::CHANNEL_COUNT
            }
        };

        #[cfg(debug_assertions)]
        {
            if frames_will_write > 0 {
                let largest_index = ((frames_will_write - 1) as f64
                    * source_frames_per_output_frame) as usize
                    * config::CHANNEL_COUNT;
                assert!(largest_index < self.source_data.len());
            }
        }

        let source = &self.source_data[self.position..];
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
