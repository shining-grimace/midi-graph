use crate::{
    AssetLoader, Error, Event, EventTarget, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Polyphony {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::max_voices")]
    pub max_voices: usize,
    pub source: NodeConfigData,
}

impl Polyphony {
    pub fn stock(inner: NodeConfigData) -> NodeConfigData {
        NodeConfigData(Box::new(Self {
            node_id: defaults::none_id(),
            max_voices: defaults::max_voices(),
            source: inner,
        }))
    }
}

impl NodeConfig for Polyphony {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let child = self.source.0.to_node(asset_loader)?;
        Ok(Box::new(PolyphonyNode::new(
            self.node_id,
            self.max_voices,
            child,
        )?))
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        Some(vec![self.source.clone()])
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

struct Voice {
    pub current_note: Option<u8>,
    pub source: GraphNode,
}

pub struct PolyphonyNode {
    node_id: u64,
    voices: Vec<Voice>,
    next_on_index: usize,
}

impl PolyphonyNode {
    pub fn new(
        node_id: Option<u64>,
        max_voices: usize,
        consumer: GraphNode,
    ) -> Result<Self, Error> {
        if max_voices < 1 {
            return Err(Error::User(format!(
                "Cannot form Polyphony with {} voices",
                max_voices
            )));
        }
        let mut voices = (0..(max_voices - 1))
            .map(|_| {
                consumer.duplicate().map(|source| Voice {
                    current_note: None,
                    source,
                })
            })
            .collect::<Result<Vec<Voice>, Error>>()?;
        voices.push(Voice {
            current_note: None,
            source: consumer,
        });
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            voices,
            next_on_index: 0,
        })
    }
}

impl Node for PolyphonyNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let voices = self
            .voices
            .iter()
            .map(|voice| {
                voice.source.duplicate().map(|source| Voice {
                    current_note: None,
                    source,
                })
            })
            .collect::<Result<Vec<Voice>, Error>>()?;
        let polyphony = Self {
            node_id: self.node_id,
            voices,
            next_on_index: 0,
        };
        Ok(Box::new(polyphony))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOn { note, .. } => {
                if let Some(index) = self
                    .voices
                    .iter()
                    .position(|voice| voice.current_note.is_none())
                {
                    let broadcast_event = Message {
                        target: EventTarget::Broadcast,
                        data: event.data.clone(),
                    };
                    self.voices[index].current_note = Some(note);
                    self.voices[index].source.on_event(&broadcast_event);
                }
                true
            }
            Event::NoteOff { note, .. } => {
                if let Some(index) = self
                    .voices
                    .iter()
                    .position(|voice| match voice.current_note {
                        Some(current_note) => current_note == note,
                        None => false,
                    })
                {
                    let broadcast_event = Message {
                        target: EventTarget::Broadcast,
                        data: event.data.clone(),
                    };
                    self.voices[index].source.on_event(&broadcast_event);
                    self.voices[index].current_note = None;
                }
                true
            }
            _ => false,
        }
    }

    fn propagate(&mut self, event: &Message) {
        for voice in self.voices.iter_mut() {
            voice.source.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for voice in self.voices.iter_mut() {
            voice.source.fill_buffer(buffer);
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User(
                "Polyphony requires one child which will be duplicated as needed".to_owned(),
            ));
        }

        self.voices = (0..(self.voices.len()))
            .map(|_| {
                children[0].duplicate().map(|source| Voice {
                    current_note: None,
                    source,
                })
            })
            .collect::<Result<Vec<Voice>, Error>>()?;
        self.next_on_index = 0;
        Ok(())
    }
}
