use crate::{
    Error, Event, GraphNode, Message, Node, NoteRange,
    abstraction::{NodeConfig, NodeConfigData, NodeRegistry, defaults},
    util,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RangeSource {
    pub source: NodeConfigData,
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

#[derive(Deserialize, Clone)]
pub struct Font {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub config: FontSource,
}

impl Font {
    pub fn stock_full_range(source: NodeConfigData) -> NodeConfigData {
        NodeConfigData(Box::new(Self {
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
    fn to_node(&self, registry: &NodeRegistry) -> Result<GraphNode, Error> {
        let node: GraphNode = match &self.config {
            FontSource::Ranges(range_configs) => {
                let mut builder = FontNodeBuilder::new(self.node_id);
                for range_config in range_configs.iter() {
                    let source = range_config.source.0.to_node(registry)?;
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
                let bytes = registry.load_asset(path)?;
                let source = util::soundfont_from_bytes(
                    self.node_id,
                    &bytes,
                    *instrument_index,
                    *polyphony_voices,
                )?;
                let source: GraphNode = Box::new(source);
                source
            }
        };
        Ok(node)
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
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
