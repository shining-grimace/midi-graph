use crate::{consts, util, BufferConsumer, Error, NoteEvent, NoteKind, Status};
use hound::{SampleFormat, WavSpec};
use soundfont::data::{sample::SampleLink, SampleHeader};

pub struct WavSource {
    source_note: u8,
    source_channel_count: usize,
    data_position: usize,
    current_note: u8,
    source_data: Vec<f32>,
    playback_scale: f64,
}

impl WavSource {
    pub fn new_from_raw_sf2_data(header: &SampleHeader, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_header(header)?;
        let source_channel_count = match header.sample_type {
            SampleLink::MonoSample => 1,
            _ => {
                return Err(Error::User(format!(
                    "SF2: Unsupported sample type: {:?}",
                    header.sample_type
                )));
            }
        };
        Ok(Self::new(
            header.sample_rate,
            source_channel_count,
            header.origpitch,
            data,
        ))
    }

    /// Make a new WavSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(spec: WavSpec, source_note: u8, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        Ok(Self::new(
            spec.sample_rate,
            spec.channels as usize,
            source_note,
            data,
        ))
    }

    fn new(sample_rate: u32, channels: usize, source_note: u8, data: Vec<f32>) -> Self {
        let playback_scale = consts::PLAYBACK_SAMPLE_RATE as f64 / sample_rate as f64;
        Self {
            source_note,
            source_channel_count: channels,
            data_position: 0,
            current_note: 0,
            source_data: data,
            playback_scale,
        }
    }

    fn validate_header(header: &SampleHeader) -> Result<(), Error> {
        match header.sample_type {
            SampleLink::MonoSample => Ok(()),
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

    fn stretch_buffer(
        src: &[f32],
        src_channels: usize,
        dst: &mut [f32],
        source_frames_per_output_frame: f64,
    ) -> (usize, usize) {
        let mut src_index = 0;
        let mut dst_index = 0;
        while src_index < src.len() && dst_index < dst.len() {
            match src_channels {
                1 => {
                    let sample = src[src_index];
                    dst[dst_index] = sample;
                    dst[dst_index + 1] = sample;
                }
                2 => {
                    dst[dst_index] = src[src_index];
                    dst[dst_index + 1] = src[src_index + 1];
                }
                _ => {}
            }
            dst_index += 2;
            src_index =
                ((dst_index / 2) as f64 * source_frames_per_output_frame) as usize * src_channels;
        }
        let uncopied_src_data_points = src.len().saturating_sub(src_index);
        let unfilled_dst_data_points = dst.len().saturating_sub(dst_index);
        (uncopied_src_data_points, unfilled_dst_data_points)
    }
}

impl BufferConsumer for WavSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        let sample_rate = (consts::PLAYBACK_SAMPLE_RATE as f64 / self.playback_scale) as u32;
        let source = Self::new(
            sample_rate,
            self.source_channel_count,
            self.source_note,
            self.source_data.clone(),
        );
        Ok(Box::new(source))
    }

    fn set_note(&mut self, event: NoteEvent) {
        match event.kind {
            NoteKind::NoteOn { note, vel: _ } => {
                self.data_position = 0;
                self.current_note = note;
            }
            NoteKind::NoteOff { note, vel: _ } => {
                if self.current_note != note {
                    return;
                }
                self.data_position = self.source_data.len();
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        // Scaling
        let relative_pitch =
            util::relative_pitch_ratio_of(self.current_note, self.source_note) as f64;
        let source_frames_per_output_frame = relative_pitch * self.playback_scale;

        let (uncopied_src_data_points, unfilled_dst_data_points) = Self::stretch_buffer(
            &self.source_data[self.data_position..],
            self.source_channel_count,
            buffer,
            source_frames_per_output_frame,
        );

        self.data_position = self.source_data.len() - uncopied_src_data_points;
        match self.data_position >= self.source_data.len() {
            true => Status::Ended,
            false => Status::Ok,
        }
    }
}
