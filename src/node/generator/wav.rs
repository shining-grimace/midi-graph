use crate::{
    AssetLoader, Balance, Error, Event, GraphNode, LoopRange, Message, Node,
    abstraction::{Loop, NodeConfig, defaults},
    consts, util,
};
use hound::{SampleFormat, WavSpec};
use serde::Deserialize;
use soundfont::raw::{SampleHeader, SampleLink};

#[derive(Deserialize, Clone)]
pub struct SampleLoop {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::source_balance")]
    pub balance: Balance,
    pub path: String,
    pub base_note: u8,
    pub looping: Option<Loop>,
}

impl NodeConfig for SampleLoop {
    fn to_node(&self, asset_loader: &Box<dyn AssetLoader>) -> Result<GraphNode, Error> {
        let loop_range = self.looping.as_ref().map(LoopRange::from_config);
        let bytes = asset_loader.load_asset_data(&self.path)?;
        let source = util::wav_from_bytes(
            &bytes,
            self.base_note,
            loop_range,
            self.balance,
            self.node_id,
        )?;
        let source: GraphNode = Box::new(source);
        Ok(source)
    }

    fn clone_child_configs(&self) -> Option<Vec<crate::abstraction::NodeConfigData>> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct SampleLoopNode {
    node_id: u64,
    is_on: bool,
    source_note: u8,
    source_channel_count: usize,
    balance: Balance,
    loop_start_data_position: usize,
    loop_end_data_position: usize,
    data_position: usize,
    current_note: u8,
    pitch_multiplier: f32,
    volume: f32,
    source_data: Vec<f32>,
    playback_scale: f64,
}

impl SampleLoopNode {
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
                    "Unsupported sample type {:?} in SF2 file (only mono is supported)",
                    header.sample_type
                )));
            }
        };
        let sample_offset = header.start as usize;
        let loop_range = LoopRange::new_frame_range(
            (header.loop_start as usize - sample_offset) / source_channel_count,
            (header.loop_end as usize - sample_offset) / source_channel_count,
        );
        Self::validate_loop_range(&data, source_channel_count, &loop_range)?;
        Ok(Self::new(
            None,
            header.sample_rate,
            source_channel_count,
            header.origpitch,
            loop_range,
            balance,
            data,
        ))
    }

    /// Make a new WavSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(
        spec: WavSpec,
        source_note: u8,
        balance: Balance,
        data: Vec<f32>,
        loop_range: Option<LoopRange>,
        node_id: Option<u64>,
    ) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        if let Some(range) = &loop_range {
            Self::validate_loop_range(&data, spec.channels as usize, range)?;
        }
        let loop_range = match loop_range {
            Some(range) => range,
            None => LoopRange::new_frame_range(0, usize::MAX / spec.channels as usize),
        };
        Ok(Self::new(
            node_id,
            spec.sample_rate,
            spec.channels as usize,
            source_note,
            loop_range,
            balance,
            data,
        ))
    }

    fn new(
        node_id: Option<u64>,
        sample_rate: u32,
        channels: usize,
        source_note: u8,
        loop_range: LoopRange,
        balance: Balance,
        data: Vec<f32>,
    ) -> Self {
        let playback_scale = consts::PLAYBACK_SAMPLE_RATE as f64 / sample_rate as f64;
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            source_note,
            source_channel_count: channels,
            balance,
            loop_start_data_position: loop_range.start_frame * channels,
            loop_end_data_position: loop_range.end_frame * channels,
            data_position: data.len(),
            current_note: 0,
            pitch_multiplier: 1.0,
            volume: 1.0,
            source_data: data,
            playback_scale,
        }
    }

    fn validate_header(header: &SampleHeader) -> Result<(), Error> {
        match header.sample_type {
            SampleLink::MonoSample => Ok(()),
            _ => Err(Error::User(format!(
                "Unsupported sample type {:?} in SF2 file (only mono is supported)",
                header.sample_type
            ))),
        }
    }

    fn validate_loop_range(
        data: &[f32],
        channel_count: usize,
        loop_range: &LoopRange,
    ) -> Result<(), Error> {
        let frames_in_data = data.len() / channel_count;
        let range_makes_sense =
            loop_range.start_frame <= frames_in_data || loop_range.end_frame > frames_in_data;
        if !range_makes_sense {
            return Err(Error::User(format!(
                "Invalid sample loop range: {} to {}",
                loop_range.start_frame, loop_range.end_frame
            )));
        }
        Ok(())
    }

    fn validate_spec(spec: &WavSpec) -> Result<(), Error> {
        if spec.channels == 0 || spec.channels > 2 {
            return Err(Error::User(format!(
                "{} channels is not supported in WAV file (only 1 or 2 is supported)",
                spec.channels
            )));
        }
        if spec.sample_format != SampleFormat::Float {
            return Err(Error::User(format!(
                "Sample format {:?} is not supported in WAV file (only 32-bit float is supported)",
                spec.sample_format
            )));
        }
        if spec.bits_per_sample != 32 {
            return Err(Error::User(format!(
                "{} bits per sample is not supported in WAV files (only 32-bit float is supported)",
                spec.bits_per_sample
            )));
        }
        Ok(())
    }

    fn stretch_buffer(
        &self,
        src: &[f32],
        src_channels: usize,
        dst: &mut [f32],
        source_frames_per_output_frame: f64,
    ) -> (usize, usize) {
        let mut src_index = 0;
        let mut dst_index = 0;
        let (left_amplitude, right_amplitude) = match self.balance {
            Balance::Both => (1.0, 1.0),
            Balance::Left => (1.0, 0.0),
            Balance::Right => (0.0, 1.0),
            Balance::Pan(pan) => (1.0 - pan, pan),
        };
        while src_index < src.len() && dst_index < dst.len() {
            match src_channels {
                1 => {
                    let sample = src[src_index] * self.volume;
                    dst[dst_index] += left_amplitude * sample;
                    dst[dst_index + 1] += right_amplitude * sample;
                }
                2 => {
                    dst[dst_index] += left_amplitude * src[src_index] * self.volume;
                    dst[dst_index + 1] += right_amplitude * src[src_index + 1] * self.volume;
                }
                _ => {}
            }
            dst_index += 2;
            src_index =
                ((dst_index / 2) as f64 * source_frames_per_output_frame) as usize * src_channels;
        }
        let src_data_points_advanced = src_index;
        let dst_data_points_advanced = dst_index;
        (src_data_points_advanced, dst_data_points_advanced)
    }
}

impl Node for SampleLoopNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let sample_rate = (consts::PLAYBACK_SAMPLE_RATE as f64 / self.playback_scale) as u32;
        let loop_range = LoopRange::new_frame_range(
            self.loop_start_data_position / self.source_channel_count,
            self.loop_end_data_position / self.source_channel_count,
        );
        let source = Self::new(
            Some(self.node_id),
            sample_rate,
            self.source_channel_count,
            self.source_note,
            loop_range,
            self.balance,
            self.source_data.clone(),
        );
        Ok(Box::new(source))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOn { note, vel: _ } => {
                self.is_on = true;
                self.data_position = 0;
                self.current_note = note;
                self.pitch_multiplier = 1.0;
            }
            Event::NoteOff { note, vel: _ } => {
                if self.current_note == note && self.is_on {
                    self.is_on = false;
                }
            }
            Event::PitchMultiplier(multiplier) => {
                self.pitch_multiplier = multiplier;
            }
            Event::SourceBalance(balance) => {
                self.balance = balance;
            }
            Event::Volume(volume) => {
                self.volume = volume;
            }
            _ => {}
        }
        true
    }

    fn propagate(&mut self, _event: &Message) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if buffer.is_empty() {
            return;
        }

        if self.is_on && self.data_position >= self.loop_end_data_position {
            self.data_position -= self.loop_end_data_position - self.loop_start_data_position;
        }

        // Scaling
        let relative_pitch = self.pitch_multiplier as f64
            * util::relative_pitch_ratio_of(self.current_note, self.source_note) as f64;
        let source_frames_per_output_frame = relative_pitch * self.playback_scale;

        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        let mut remaining_buffer = &mut buffer[0..];
        while !remaining_buffer.is_empty() {
            if self.data_position >= self.source_data.len() {
                self.is_on = false;
                return;
            }

            let source_end_point = match self.is_on {
                true => self.source_data.len().min(self.loop_end_data_position),
                false => self.source_data.len(),
            };

            let (src_data_points_advanced, dst_data_points_advanced) = self.stretch_buffer(
                &self.source_data[self.data_position..source_end_point],
                self.source_channel_count,
                remaining_buffer,
                source_frames_per_output_frame,
            );

            self.data_position += src_data_points_advanced;

            if self.data_position != source_end_point {
                break;
            }
            if self.is_on && source_end_point == self.loop_end_data_position {
                self.data_position = self.loop_start_data_position;
                let remaining_dst_data_points = remaining_buffer.len() - dst_data_points_advanced;
                let dst_buffer_index = buffer.len() - remaining_dst_data_points;
                remaining_buffer = &mut buffer[dst_buffer_index..];
            } else {
                self.is_on = false;
                return;
            }
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User("WavSource cannot have children".to_owned())),
        }
    }
}
