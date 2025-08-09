use super::util as font_util;
use crate::{
    AssetLoadPayload, AssetLoader, Balance, DebugLogging, Error, Event, GraphNode, LoopRange,
    Message, Node, NoteRange, SampleBuffer,
    abstraction::{ChildConfig, NodeConfig, defaults},
    generator::SampleLoopNode,
    group::PolyphonyNode,
};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use soundfont::{SoundFont2, data::SampleLink};
use std::{
    io::{Cursor, Seek, SeekFrom},
    sync::Arc,
};

#[derive(Deserialize, Clone)]
pub struct RangeSource {
    pub source: ChildConfig,
    pub lower: u8,
    pub upper: u8,
}

#[derive(Deserialize, Clone)]
pub enum FontSource {
    Ranges(Vec<RangeSource>),
    Sf2FilePath {
        path: String,
        instrument_index: usize,
        #[serde(default = "defaults::soundfont_polyphony_voices")]
        polyphony_voices: usize,
    },
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FontFileMetadata {
    pub instruments: Vec<InstrumentMetadata>,
}

impl FontFileMetadata {
    fn from_spec(sf2: &SoundFont2) -> Result<Self, Error> {
        let mut instruments: Vec<InstrumentMetadata> = vec![];
        for instrument in sf2.instruments.iter() {
            let mut ranges: Vec<InstrumentRangeMetadata> = vec![];
            for zone in instrument.zones.iter() {
                let Some(sample_index) = zone.sample() else {
                    println!("WARNING: SF2: Sample index not found for instrument zone");
                    continue;
                };
                let Some(sample_header) = sf2.sample_headers.get(*sample_index as usize) else {
                    println!(
                        "WARNING: SF2: Sample index {} not found matching instrument zone",
                        sample_index
                    );
                    continue;
                };
                let channel_count = match sample_header.sample_type {
                    SampleLink::MonoSample => 1,
                    _ => {
                        return Err(Error::User(format!(
                            "Unsupported sample type for SF2 files: {:?}",
                            sample_header.sample_type
                        )));
                    }
                };

                let data_offset = sample_header.start as usize;
                let sample_count = sample_header.end as usize - data_offset;
                let note_range = font_util::note_range_for_zone(zone)?;
                let loop_range = LoopRange::new_frame_range(
                    (sample_header.loop_start as usize - data_offset) / channel_count,
                    (sample_header.loop_end as usize - data_offset) / channel_count,
                );
                ranges.push(InstrumentRangeMetadata {
                    note_range,
                    channel_count,
                    sample_rate: sample_header.sample_rate,
                    base_note: sample_header.origpitch,
                    loop_range,
                    buffer_index: data_offset,
                    buffer_length: sample_count,
                });
            }
            instruments.push(InstrumentMetadata { ranges });
        }
        Ok(Self { instruments })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct InstrumentMetadata {
    pub ranges: Vec<InstrumentRangeMetadata>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct InstrumentRangeMetadata {
    pub note_range: NoteRange,
    pub channel_count: usize,
    pub sample_rate: u32,
    pub base_note: u8,
    pub loop_range: LoopRange,
    pub buffer_index: usize,
    pub buffer_length: usize,
}

#[derive(Deserialize, Clone)]
pub struct Font {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub config: FontSource,
}

impl Font {
    pub fn stock_full_range(source: ChildConfig) -> ChildConfig {
        ChildConfig(Box::new(Self {
            node_id: defaults::none_id(),
            config: FontSource::Ranges(vec![RangeSource {
                source,
                lower: 0,
                upper: 127,
            }]),
        }))
    }
}

impl NodeConfig for Font {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let node: GraphNode = match &self.config {
            FontSource::Ranges(range_configs) => {
                let mut builder = FontNodeBuilder::new(self.node_id);
                for range_config in range_configs.iter() {
                    let source = range_config.source.0.to_node(asset_loader)?;
                    let range = NoteRange::from_config(range_config);
                    builder = builder.add_range(range, source)?;
                }
                Box::new(builder.build())
            }
            FontSource::Sf2FilePath {
                path,
                instrument_index,
                polyphony_voices,
            } => {
                let (metadata, sample_buffer) = match asset_loader.load_asset_data(path)? {
                    AssetLoadPayload::RawAssetData(raw_data) => {
                        let mut cursor = Cursor::new(raw_data.as_slice());
                        let sf2 = SoundFont2::load(&mut cursor)?;
                        font_util::validate_sf2_file(&sf2)?;

                        if DebugLogging::get_log_on_init() {
                            font_util::log_opened_sf2(&sf2);
                        }

                        let sample_chunk_metadata = &sf2.sample_data.smpl.ok_or_else(|| {
                            Error::User("There was no sample header in the SF2 file".to_owned())
                        })?;

                        let data_point_size = std::mem::size_of::<i16>();
                        cursor.seek(SeekFrom::Start(sample_chunk_metadata.offset as u64))?;
                        let mut sample_data =
                            vec![0i16; sample_chunk_metadata.len as usize / data_point_size];
                        cursor.read_i16_into::<LittleEndian>(&mut sample_data)?;

                        let float_buffer = sample_data
                            .into_iter()
                            .map(|s| s as f32 / 32768.0)
                            .collect();
                        let sample_buffer: SampleBuffer = Arc::new(float_buffer);

                        let metadata = FontFileMetadata::from_spec(&sf2)?;
                        let raw_metadata = Arc::new(serde_json::to_vec(&metadata)?);
                        asset_loader.store_prepared_data(
                            path,
                            raw_metadata.clone(),
                            sample_buffer.clone(),
                        );
                        (metadata, sample_buffer)
                    }
                    AssetLoadPayload::PreparedData((raw_metadata, sample_buffer)) => {
                        let metadata: FontFileMetadata = serde_json::from_slice(&raw_metadata)?;
                        (metadata, sample_buffer)
                    }
                };

                let Some(instrument) = metadata.instruments.get(*instrument_index) else {
                    return Err(Error::User(format!(
                        "Index {} is out of bounds ({} instruments in the SF2 file)",
                        instrument_index,
                        metadata.instruments.len()
                    )));
                };

                let mut soundfont_builder = FontNodeBuilder::new(self.node_id);
                for range in instrument.ranges.iter() {
                    let source = SampleLoopNode::new(
                        None,
                        range.sample_rate,
                        range.channel_count,
                        range.base_note,
                        Some(range.loop_range.clone()),
                        Balance::Both,
                        sample_buffer.clone(),
                        range.buffer_index as usize,
                        range.buffer_length as usize,
                    )?;
                    let polyphony: GraphNode = match polyphony_voices {
                        0 | 1 => {
                            let polyphony =
                                PolyphonyNode::new(None, *polyphony_voices, Box::new(source))?;
                            Box::new(polyphony)
                        }
                        _ => Box::new(source),
                    };

                    soundfont_builder =
                        soundfont_builder.add_range(range.note_range.clone(), polyphony)?;
                }

                let source = soundfont_builder.build();
                let source: GraphNode = Box::new(source);
                source
            }
        };
        Ok(node)
    }

    fn clone_child_configs(&self) -> Option<Vec<ChildConfig>> {
        match &self.config {
            FontSource::Ranges(range_sources) => {
                let sources = range_sources
                    .iter()
                    .map(|range_source| range_source.source.clone())
                    .collect();
                Some(sources)
            }
            FontSource::Sf2FilePath { .. } => None,
        }
    }

    fn asset_source(&self) -> Option<&str> {
        match &self.config {
            FontSource::Ranges(_) => None,
            FontSource::Sf2FilePath { path, .. } => Some(path),
        }
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct FontNodeBuilder {
    node_id: Option<u64>,
    ranges: Vec<(NoteRange, GraphNode)>,
}

impl Default for FontNodeBuilder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FontNodeBuilder {
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id,
            ranges: vec![],
        }
    }

    pub fn add_range(mut self, range: NoteRange, consumer: GraphNode) -> Result<Self, Error> {
        self.ranges.push((range, consumer));
        Ok(self)
    }

    pub fn build(self) -> FontNode {
        FontNode::new(self.node_id, self.ranges)
    }
}

pub struct FontNode {
    node_id: u64,
    ranges: Vec<(NoteRange, GraphNode)>,
}

impl FontNode {
    fn new(node_id: Option<u64>, ranges: Vec<(NoteRange, GraphNode)>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            ranges,
        }
    }
}

impl Node for FontNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        Err(Error::User("SoundFont cannot be duplicated".to_owned()))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        let note = match event.data {
            Event::NoteOn { note, .. } => Some(note),
            Event::NoteOff { note, .. } => Some(note),
            _ => None,
        };
        if note.is_some() {
            let note = note.unwrap();
            for (range, consumer) in self.ranges.iter_mut() {
                if !range.contains(note) {
                    continue;
                }
                consumer.on_event(event);
            }
        } else {
            for (_, consumer) in self.ranges.iter_mut() {
                consumer.on_event(event);
            }
        }
        true
    }

    fn propagate(&mut self, event: &Message) {
        for (_, consumer) in self.ranges.iter_mut() {
            consumer.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for (_, consumer) in self.ranges.iter_mut() {
            consumer.fill_buffer(buffer);
        }
    }

    fn replace_children(&mut self, _children: &[GraphNode]) -> Result<(), Error> {
        Err(Error::User(
            "SoundFont does not support replacing its children".to_owned(),
        ))
    }
}
