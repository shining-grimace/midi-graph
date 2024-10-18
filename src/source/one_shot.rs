use crate::{
    consts, BufferConsumer, BufferConsumerNode, ControlEvent, Error, Node, NodeEvent, NoteEvent,
};
use hound::{SampleFormat, WavSpec};
use soundfont::data::{sample::SampleLink, SampleHeader};

pub struct OneShotSource {
    node_id: u64,
    source_channel_count: usize,
    volume: f32,
    data_position: usize,
    source_data: Vec<f32>,
}

impl OneShotSource {
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
        Ok(Self::new(None, source_channel_count, data))
    }

    /// Make a new OneShotSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(
        spec: WavSpec,
        data: Vec<f32>,
        node_id: Option<u64>,
    ) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        Ok(Self::new(node_id, spec.channels as usize, data))
    }

    fn new(node_id: Option<u64>, channels: usize, data: Vec<f32>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(|| <Self as Node>::new_node_id()),
            source_channel_count: channels,
            volume: 1.0,
            data_position: data.len(),
            source_data: data,
        }
    }

    fn validate_header(header: &SampleHeader) -> Result<(), Error> {
        if header.sample_rate as usize != consts::PLAYBACK_SAMPLE_RATE {
            println!(
                "WARNING: SF2: Sample rate {} should match playback rate of {}",
                header.sample_rate,
                consts::PLAYBACK_SAMPLE_RATE
            );
        }
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
        if spec.sample_rate as usize != consts::PLAYBACK_SAMPLE_RATE {
            println!(
                "WARNING: SF2: Sample rate {} should match playback rate of {}",
                spec.sample_rate,
                consts::PLAYBACK_SAMPLE_RATE
            );
        }
        Ok(())
    }
}

impl BufferConsumerNode for OneShotSource {}

impl Node for OneShotSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, event: &NodeEvent) {
        match event {
            NodeEvent::Note { note: _, event } => match event {
                NoteEvent::NoteOn { vel: _ } => {
                    self.data_position = 0;
                }
                NoteEvent::NoteOff { vel: _ } => {
                    self.data_position = self.source_data.len();
                }
            },
            NodeEvent::Control {
                node_id,
                event: ControlEvent::Volume(volume),
            } => {
                if *node_id != self.node_id {
                    return;
                }
                self.volume = *volume;
            }
            _ => {}
        }
    }
}

impl BufferConsumer for OneShotSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let source = Self::new(
            Some(self.node_id),
            self.source_channel_count,
            self.source_data.clone(),
        );
        Ok(Box::new(source))
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if buffer.is_empty() {
            return;
        }

        if self.data_position >= self.source_data.len() {
            return;
        }

        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        let src = &self.source_data[self.data_position..];
        match self.source_channel_count {
            1 => {
                let src_data_points = (buffer.len() / 2).min(src.len());
                for src_data_index in 0..src_data_points {
                    let sample = src[src_data_index] * self.volume;
                    buffer[src_data_index * 2] += sample;
                    buffer[src_data_index * 2 + 1] += sample;
                }
                self.data_position += src_data_points;
            }
            2 => {
                let src_data_points = buffer.len().min(src.len());
                for src_data_index in 0..src_data_points {
                    buffer[src_data_index] += src[src_data_index] * self.volume;
                }
                self.data_position += src_data_points;
            }
            _ => {}
        }
    }
}
