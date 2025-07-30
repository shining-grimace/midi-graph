use crate::{
    AssetLoadPayload, AssetLoader, Balance, Error, Event, GraphNode, Message, Node, SampleBuffer,
    abstraction::{NodeConfig, defaults},
    consts,
};
use hound::{SampleFormat, WavReader, WavSpec};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Arc};

#[derive(Deserialize, Serialize, Clone)]
pub struct OneShotFileMetadata {
    pub channels: usize,
}

impl OneShotFileMetadata {
    pub fn from_spec(spec: WavSpec) -> Result<Self, Error> {
        OneShotNode::validate_spec(&spec)?;
        Ok(Self {
            channels: spec.channels as usize,
        })
    }
}

#[derive(Deserialize, Clone)]
pub struct OneShot {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::source_balance")]
    pub balance: Balance,
    pub path: String,
}

impl NodeConfig for OneShot {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let (metadata, sample_buffer) = match asset_loader.load_asset_data(&self.path)? {
            AssetLoadPayload::RawAssetData(raw_data) => {
                let cursor = Cursor::new(raw_data);
                let wav = WavReader::new(cursor)?;
                let spec = wav.spec();
                let metadata = OneShotFileMetadata::from_spec(spec)?;
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
                let metadata: OneShotFileMetadata = serde_json::from_slice(&raw_metadata)?;
                (metadata, sample_buffer)
            }
        };
        let source =
            OneShotNode::new_from_data(self.node_id, self.balance, metadata, sample_buffer)?;
        let source: GraphNode = Box::new(source);
        Ok(source)
    }

    fn clone_child_configs(&self) -> Option<Vec<crate::abstraction::NodeConfigData>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        Some(&self.path)
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct OneShotNode {
    node_id: u64,
    source_channel_count: usize,
    balance: Balance,
    volume: f32,
    data_position: usize,
    sample_buffer: SampleBuffer,
}

impl OneShotNode {
    pub fn new_from_data(
        node_id: Option<u64>,
        balance: Balance,
        file_metadata: OneShotFileMetadata,
        sample_buffer: SampleBuffer,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            node_id,
            file_metadata.channels as usize,
            balance,
            sample_buffer,
        ))
    }

    fn new(
        node_id: Option<u64>,
        channels: usize,
        balance: Balance,
        sample_buffer: SampleBuffer,
    ) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            source_channel_count: channels,
            balance,
            volume: 1.0,
            data_position: sample_buffer.len(),
            sample_buffer,
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

impl Node for OneShotNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let source = Self::new(
            Some(self.node_id),
            self.source_channel_count,
            self.balance,
            self.sample_buffer.clone(),
        );
        Ok(Box::new(source))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOn { .. } => {
                self.data_position = 0;
            }
            Event::NoteOff { .. } => {
                self.data_position = self.sample_buffer.len();
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

        if self.data_position >= self.sample_buffer.len() {
            return;
        }

        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        let src = &self.sample_buffer[self.data_position..];
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
                    buffer[src_data_index + 1] +=
                        right_amplitude * src[src_data_index + 1] * self.volume;
                }
                self.data_position += src_data_points;
            }
            _ => {}
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User("OneShotSource cannot have children".to_owned())),
        }
    }
}
