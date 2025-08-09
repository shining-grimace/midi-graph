use crate::{
    AssetLoader, Error, GraphNode, Message, Node,
    abstraction::{ChildConfig, NodeConfig, defaults},
    consts,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Combiner {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub sources: Vec<ChildConfig>,
}

impl NodeConfig for Combiner {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let children_nodes = self
            .sources
            .iter()
            .map(|config| config.0.to_node(asset_loader))
            .collect::<Result<Vec<GraphNode>, Error>>()?;
        Ok(Box::new(CombinerNode::new(self.node_id, children_nodes)))
    }

    fn clone_child_configs(&self) -> Option<Vec<ChildConfig>> {
        Some(self.sources.clone())
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct CombinerNode {
    node_id: u64,
    consumers: Vec<GraphNode>,
    intermediate_buffer: Vec<f32>,
}

impl CombinerNode {
    pub fn new(node_id: Option<u64>, consumers: Vec<GraphNode>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            consumers,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl Node for CombinerNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumers: Result<Vec<GraphNode>, Error> =
            self.consumers.iter().map(|c| c.duplicate()).collect();
        let combiner = Self::new(Some(self.node_id), consumers?);
        Ok(Box::new(combiner))
    }

    fn try_consume_event(&mut self, _event: &Message) -> bool {
        false
    }

    fn propagate(&mut self, event: &Message) {
        for consumer in self.consumers.iter_mut() {
            consumer.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let buffer_size = buffer.len();
        let sample_count = buffer_size / consts::CHANNEL_COUNT;
        let intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        for consumer in self.consumers.iter_mut() {
            intermediate_slice.fill(0.0);
            consumer.fill_buffer(intermediate_slice);
            for i in 0..sample_count {
                let index = i * 2;
                buffer[index] += intermediate_slice[index];
                buffer[index + 1] += intermediate_slice[index + 1];
            }
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        self.consumers = children
            .iter()
            .map(|child| child.duplicate())
            .collect::<Result<Vec<GraphNode>, Error>>()?;
        Ok(())
    }
}
