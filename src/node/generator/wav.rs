use crate::{
    AssetLoadPayload, AssetLoader, Balance, Error, Event, GraphNode, LoopRange, Message, Node,
    SampleBuffer,
    abstraction::{Loop, NodeConfig, defaults},
    consts, util,
};
use hound::{SampleFormat, WavReader, WavSpec};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Arc};

#[derive(Deserialize, Serialize, Clone)]
pub struct SampleLoopFileMetadata {
    sample_rate: u32,
    channels: usize,
}

impl SampleLoopFileMetadata {
    pub fn from_spec(spec: WavSpec) -> Self {
        Self {
            sample_rate: spec.sample_rate,
            channels: spec.channels as usize,
        }
    }
}

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
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let (metadata, sample_buffer) = match asset_loader.load_asset_data(&self.path)? {
            AssetLoadPayload::RawAssetData(raw_data) => {
                let cursor = Cursor::new(raw_data);
                let wav = WavReader::new(cursor)?;
                let spec = wav.spec();
                SampleLoopNode::validate_spec(&spec)?;
                let metadata = SampleLoopFileMetadata::from_spec(spec);
                let data: Vec<f32> = wav.into_samples().map(|s| s.unwrap()).collect();
                let sample_buffer = Arc::new(data);
                let raw_metadata = Arc::new(serde_json::to_vec(&metadata)?);
                asset_loader.store_prepared_data(
                    &self.path,
                    raw_metadata.clone(),
                    sample_buffer.clone(),
                );
                (metadata, sample_buffer)
            }
            AssetLoadPayload::PreparedData((raw_metadata, sample_buffer)) => {
                let metadata: SampleLoopFileMetadata = serde_json::from_slice(&raw_metadata)?;
                (metadata, sample_buffer)
            }
        };
        let loop_range = self.looping.as_ref().map(LoopRange::from_config);
        let source = SampleLoopNode::new_from_data(
            self.node_id,
            self.balance,
            self.base_note,
            loop_range,
            metadata,
            sample_buffer,
        )?;
        let source: GraphNode = Box::new(source);
        Ok(source)
    }

    fn clone_child_configs(&self) -> Option<Vec<crate::abstraction::ChildConfig>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        Some(&self.path)
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
    loop_start_buffer_index: usize,
    loop_end_buffer_index: usize,
    data_position: usize,
    current_note: u8,
    pitch_multiplier: f32,
    volume: f32,
    sample_buffer: SampleBuffer,
    buffer_start_index: usize,
    buffer_length_samples: usize,
    playback_scale: f64,
}

impl SampleLoopNode {
    /// Make a new WavSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(
        node_id: Option<u64>,
        balance: Balance,
        source_note: u8,
        loop_range: Option<LoopRange>,
        metadata: SampleLoopFileMetadata,
        sample_buffer: SampleBuffer,
    ) -> Result<Self, Error> {
        if let Some(range) = &loop_range {
            Self::validate_loop_range(&sample_buffer, metadata.channels, range)?;
        }
        let buffer_length = sample_buffer.len();
        Ok(Self::new(
            node_id,
            metadata.sample_rate,
            metadata.channels,
            source_note,
            loop_range,
            balance,
            sample_buffer,
            0,
            buffer_length,
        )?)
    }

    pub fn new(
        node_id: Option<u64>,
        sample_rate: u32,
        channels: usize,
        source_note: u8,
        loop_range: Option<LoopRange>,
        balance: Balance,
        sample_buffer: SampleBuffer,
        buffer_start_index: usize,
        buffer_length_samples: usize,
    ) -> Result<Self, Error> {
        let playback_scale = consts::PLAYBACK_SAMPLE_RATE as f64 / sample_rate as f64;
        if sample_buffer.len() < buffer_start_index + buffer_length_samples {
            return Err(Error::User(format!(
                "ERROR: WAV: Buffer of size {} too small for sample of size {} at index {}",
                sample_buffer.len(),
                buffer_length_samples,
                buffer_start_index
            )));
        }
        if let Some(looping) = &loop_range {
            if looping.start_frame >= looping.end_frame {
                return Err(Error::User(format!(
                    "ERROR: WAV: Loop start {} must be before end {}",
                    looping.start_frame, looping.end_frame
                )));
            }
            if looping.end_frame * channels > buffer_length_samples {
                return Err(Error::User(format!(
                    "ERROR: WAV: Loop end index {} cannot be greater than buffer size {}",
                    looping.end_frame * channels,
                    buffer_length_samples
                )));
            }
        }
        let buffer_loop_start_index = loop_range.as_ref().map_or(0, |looping| {
            buffer_start_index + looping.start_frame * channels
        });
        let buffer_loop_end_index = loop_range.as_ref().map_or(usize::MAX, |looping| {
            buffer_start_index + looping.end_frame * channels
        });
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            source_note,
            source_channel_count: channels,
            balance,
            loop_start_buffer_index: buffer_loop_start_index,
            loop_end_buffer_index: buffer_loop_end_index,
            data_position: sample_buffer.len(),
            current_note: 0,
            pitch_multiplier: 1.0,
            volume: 1.0,
            sample_buffer,
            buffer_start_index,
            buffer_length_samples,
            playback_scale,
        })
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
        let loop_range = match self.loop_end_buffer_index == usize::MAX {
            true => None,
            false => Some(LoopRange::new_frame_range(
                self.loop_start_buffer_index / self.source_channel_count,
                self.loop_end_buffer_index / self.source_channel_count,
            )),
        };
        let source = Self::new(
            Some(self.node_id),
            sample_rate,
            self.source_channel_count,
            self.source_note,
            loop_range,
            self.balance,
            self.sample_buffer.clone(),
            self.buffer_start_index,
            self.buffer_length_samples,
        )?;
        Ok(Box::new(source))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOn { note, vel: _ } => {
                self.is_on = true;
                self.data_position = self.buffer_start_index;
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

        if self.is_on && self.data_position >= self.loop_end_buffer_index {
            self.data_position -= self.loop_end_buffer_index - self.loop_start_buffer_index;
        }

        // Scaling
        let relative_pitch = self.pitch_multiplier as f64
            * util::relative_pitch_ratio_of(self.current_note, self.source_note) as f64;
        let source_frames_per_output_frame = relative_pitch * self.playback_scale;

        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        let mut remaining_buffer = &mut buffer[0..];
        while !remaining_buffer.is_empty() {
            let sample_end_index = self.buffer_start_index + self.buffer_length_samples;
            if self.data_position >= sample_end_index {
                self.is_on = false;
                return;
            }

            let source_end_point = match self.is_on {
                true => sample_end_index.min(self.loop_end_buffer_index),
                false => sample_end_index,
            };

            let (src_data_points_advanced, dst_data_points_advanced) = self.stretch_buffer(
                &self.sample_buffer[self.data_position..source_end_point],
                self.source_channel_count,
                remaining_buffer,
                source_frames_per_output_frame,
            );

            self.data_position += src_data_points_advanced;

            if self.data_position != source_end_point {
                break;
            }
            if self.is_on && source_end_point == self.loop_end_buffer_index {
                self.data_position = self.loop_start_buffer_index;
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
