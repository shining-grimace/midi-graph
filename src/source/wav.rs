use crate::{config, util, BufferConsumer, Error, NoteEvent};
use hound::{SampleFormat, WavSpec};
use soundfont::data::{sample::SampleLink, SampleHeader};

pub struct WavSource {
    source_note: u8,
    source_channel_count: usize,
    position: usize,
    current_note: u8,
    source_data: Vec<f32>,
    playback_scale: f64,
}

impl WavSource {
    pub fn new_from_raw_data(header: &SampleHeader, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_header(header)?;
        let playback_scale = config::PLAYBACK_SAMPLE_RATE as f64 / header.sample_rate as f64;
        let source_channel_count = match header.sample_type {
            SampleLink::MonoSample => 1,
            SampleLink::LinkedSample => 2,
            _ => {
                return Err(Error::User(format!(
                    "SF2: Unsupported sample type: {:?}",
                    header.sample_type
                )));
            }
        };
        Ok(Self {
            source_note: header.origpitch,
            source_channel_count,
            position: 0,
            current_note: 0,
            source_data: data,
            playback_scale,
        })
    }

    /// Make a new WavSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(spec: WavSpec, source_note: u8, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        let playback_scale = config::PLAYBACK_SAMPLE_RATE as f64 / spec.sample_rate as f64;
        Ok(Self {
            source_note,
            source_channel_count: spec.channels as usize,
            position: 0,
            current_note: 0,
            source_data: data,
            playback_scale,
        })
    }

    fn validate_header(header: &SampleHeader) -> Result<(), Error> {
        match header.sample_type {
            SampleLink::MonoSample => Ok(()),
            SampleLink::LinkedSample => Ok(()),
            _ => Err(Error::User(format!(
                "SF2: Unsupported sample type: {:?}",
                header.sample_type
            ))),
        }
    }

    fn validate_spec(spec: &WavSpec) -> Result<(), Error> {
        if spec.channels == 0 || spec.channels > 2 {
            return Err(Error::User(format!(
                "{} channels is not supported",
                spec.channels
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

        // Scaling
        let relative_pitch =
            util::relative_pitch_ratio_of(self.current_note, self.source_note) as f64;
        let source_frames_per_output_frame = relative_pitch * self.playback_scale;

        // Output properties
        let samples_can_write = buffer.len();
        let frames_can_write = samples_can_write / config::CHANNEL_COUNT;

        // Input properties
        let source_samples_remaining = self.source_data.len() - self.position;
        let source_frames_remaining = source_samples_remaining / self.source_channel_count;

        // Transfer alignment
        let needed_source_frames = {
            let unrounded = (frames_can_write as f64 * source_frames_per_output_frame) as usize;
            unrounded - (unrounded % self.source_channel_count)
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
                    * self.source_channel_count;
                assert!(largest_index < self.source_data.len());
            }
        }

        let source = &self.source_data[self.position..];
        for i in 0..frames_will_write {
            let output_index = i * config::CHANNEL_COUNT;
            match self.source_channel_count {
                1 => {
                    let source_index = (i as f64 * source_frames_per_output_frame) as usize;
                    buffer[output_index] += source[source_index];
                    buffer[output_index + 1] += source[source_index];
                }
                2 => {
                    let source_index = (i as f64 * source_frames_per_output_frame) as usize * 2;
                    buffer[output_index] += source[source_index];
                    buffer[output_index + 1] += source[source_index + 1];
                }
                _ => {}
            };
        }

        let frames_did_read = needed_source_frames.min(source_frames_remaining);
        self.position = (self.position + (frames_did_read * self.source_channel_count))
            .min(self.source_data.len());
    }
}
