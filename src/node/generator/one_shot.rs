use crate::{Balance, Error, Event, Message, Node, consts};
use hound::{SampleFormat, WavSpec};
use soundfont::raw::{SampleHeader, SampleLink};

pub struct OneShotSource {
    node_id: u64,
    source_channel_count: usize,
    balance: Balance,
    volume: f32,
    data_position: usize,
    source_data: Vec<f32>,
}

impl OneShotSource {
    pub fn new_from_raw_sf2_data(
        header: &SampleHeader,
        balance: Balance,
        data: Vec<f32>,
    ) -> Result<Self, Error> {
        Self::validate_header(header)?;
        let source_channel_count = match header.sample_type {
            SampleLink::MonoSample => 1,
            _ => {
                return Err(Error::User(format!(
                    "Unsupported sample type for SF2 files: {:?}",
                    header.sample_type
                )));
            }
        };
        Ok(Self::new(None, source_channel_count, balance, data))
    }

    /// Make a new OneShotSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(
        spec: WavSpec,
        balance: Balance,
        data: Vec<f32>,
        node_id: Option<u64>,
    ) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        Ok(Self::new(node_id, spec.channels as usize, balance, data))
    }

    fn new(node_id: Option<u64>, channels: usize, balance: Balance, data: Vec<f32>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            source_channel_count: channels,
            balance,
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
                "Unsupported sample type for SF2 files: {:?}",
                header.sample_type
            ))),
        }
    }

    fn validate_spec(spec: &WavSpec) -> Result<(), Error> {
        if spec.channels == 0 || spec.channels > 2 {
            return Err(Error::User(format!(
                "{} channels is not supported for WAV files (only 1 or 2 is supported)",
                spec.channels
            )));
        }
        if spec.sample_format != SampleFormat::Float {
            return Err(Error::User(format!(
                "Sample format {:?} is not supported for WAV files (only 32-bit float is supported)",
                spec.sample_format
            )));
        }
        if spec.bits_per_sample != 32 {
            return Err(Error::User(format!(
                "{} bits per sample is not supported for WAV files (only 32-bit float is supported)",
                spec.bits_per_sample
            )));
        }
        if spec.sample_rate as usize != consts::PLAYBACK_SAMPLE_RATE {
            println!(
                "WARNING: (WAV) Sample rate {} should match playback rate of {}",
                spec.sample_rate,
                consts::PLAYBACK_SAMPLE_RATE
            );
        }
        Ok(())
    }
}

impl Node for OneShotSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let source = Self::new(
            Some(self.node_id),
            self.source_channel_count,
            self.balance,
            self.source_data.clone(),
        );
        Ok(Box::new(source))
    }

    fn on_event(&mut self, event: &Message) {
        if !event.target.influences(self.node_id) {
            return;
        }
        match event.data {
            Event::NoteOn { .. } => {
                self.data_position = 0;
            }
            Event::NoteOff { .. } => {
                self.data_position = self.source_data.len();
            }
            Event::SourceBalance(balance) => {
                self.balance = balance;
            }
            Event::Volume(volume) => {
                self.volume = volume;
            }
            _ => {}
        }
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
        let (left_amplitude, right_amplitude) = match self.balance {
            Balance::Both => (1.0, 1.0),
            Balance::Left => (1.0, 0.0),
            Balance::Right => (0.0, 1.0),
            Balance::Pan(pan) => (1.0 - pan, pan),
        };
        match self.source_channel_count {
            1 => {
                let src_data_points = (buffer.len() / 2).min(src.len());
                for src_data_index in 0..src_data_points {
                    let sample = src[src_data_index] * self.volume;
                    buffer[src_data_index * 2] += left_amplitude * sample;
                    buffer[src_data_index * 2 + 1] += right_amplitude * sample;
                }
                self.data_position += src_data_points;
            }
            2 => {
                let src_data_points = buffer.len().min(src.len());
                for src_data_index in (0..src_data_points).step_by(2) {
                    buffer[src_data_index] += left_amplitude * src[src_data_index] * self.volume;
                    buffer[src_data_index + 1] += right_amplitude * src[src_data_index + 1] * self.volume;
                }
                self.data_position += src_data_points;
            }
            _ => {}
        }
    }

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User("OneShotSource cannot have children".to_owned())),
        }
    }
}
