use crate::{
    AssetLoader, Error, Event, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
    consts,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Mixer {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::mixer_balance")]
    pub balance: f32,
    pub sources: [NodeConfigData; 2],
}

impl Mixer {
    pub fn stock(inner_0: NodeConfigData, inner_1: NodeConfigData) -> NodeConfigData {
        NodeConfigData(Box::new(Self {
            node_id: defaults::none_id(),
            balance: defaults::mixer_balance(),
            sources: [inner_0, inner_1],
        }))
    }
}

impl NodeConfig for Mixer {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let consumer_0 = self.sources[0].0.to_node(asset_loader)?;
        let consumer_1 = self.sources[1].0.to_node(asset_loader)?;
        Ok(Box::new(MixerNode::new(
            self.node_id,
            self.balance,
            consumer_0,
            consumer_1,
        )))
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        Some(vec![self.sources[0].clone(), self.sources[1].clone()])
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct MixerNode {
    node_id: u64,
    balance: f32,
    consumer_0: GraphNode,
    consumer_1: GraphNode,
    intermediate_buffer: Vec<f32>,
}

impl MixerNode {
    pub fn new(
        node_id: Option<u64>,
        balance: f32,
        consumer_0: GraphNode,
        consumer_1: GraphNode,
    ) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            balance,
            consumer_0,
            consumer_1,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl Node for MixerNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumer_0 = self.consumer_0.duplicate()?;
        let consumer_1 = self.consumer_1.duplicate()?;
        let mixer = Self::new(Some(self.node_id), self.balance, consumer_0, consumer_1);
        Ok(Box::new(mixer))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        if let Message {
            data: Event::MixerBalance(balance),
            ..
        } = event
        {
            self.balance = *balance;
            true
        } else {
            false
        }
    }

    fn propagate(&mut self, event: &Message) {
        self.consumer_0.on_event(event);
        self.consumer_1.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let buffer_size = buffer.len();
        let sample_count = buffer_size / consts::CHANNEL_COUNT;
        let intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        intermediate_slice.fill(0.0);
        self.consumer_0.fill_buffer(intermediate_slice);
        let multiplier_0 = 1.0 - self.balance;
        for i in 0..sample_count {
            let index = i * 2;
            buffer[index] += multiplier_0 * intermediate_slice[index];
            buffer[index + 1] += multiplier_0 * intermediate_slice[index + 1];
        }
        intermediate_slice.fill(0.0);
        self.consumer_1.fill_buffer(intermediate_slice);
        for i in 0..sample_count {
            let index = i * 2;
            buffer[index] += self.balance * intermediate_slice[index];
            buffer[index + 1] += self.balance * intermediate_slice[index + 1];
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 2 {
            return Err(Error::User("Mixer requires two children".to_owned()));
        }
        self.consumer_0 = children[0].duplicate()?;
        self.consumer_1 = children[1].duplicate()?;
        Ok(())
    }
}
