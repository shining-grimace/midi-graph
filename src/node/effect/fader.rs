use crate::{
    AssetLoader, Error, Event, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
    consts,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Fader {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub initial_volume: f32,
    pub source: NodeConfigData,
}

impl Fader {
    pub fn stock(inner: NodeConfigData) -> NodeConfigData {
        NodeConfigData(Box::new(Self {
            node_id: defaults::none_id(),
            initial_volume: 1.0,
            source: inner,
        }))
    }
}

impl NodeConfig for Fader {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let child_node = self.source.0.to_node(asset_loader)?;
        Ok(Box::new(FaderNode::new(
            self.node_id,
            self.initial_volume,
            child_node,
        )))
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

pub struct FaderNode {
    node_id: u64,
    duration_seconds: f32,
    from_volume: f32,
    to_volume: f32,
    progress_seconds: f32,
    consumer: GraphNode,
    intermediate_buffer: Vec<f32>,
}

impl FaderNode {
    pub fn new(node_id: Option<u64>, initial_volume: f32, consumer: GraphNode) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            duration_seconds: 0.0,
            from_volume: initial_volume,
            to_volume: initial_volume,
            progress_seconds: 0.0,
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl Node for FaderNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumer = self.consumer.duplicate()?;
        let fader = Self {
            node_id: self.node_id,
            duration_seconds: self.duration_seconds,
            from_volume: self.from_volume,
            to_volume: self.to_volume,
            progress_seconds: self.progress_seconds,
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        };
        Ok(Box::new(fader))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        if let Event::Fade { from, to, seconds } = event.data {
            self.from_volume = from;
            self.to_volume = to;
            self.duration_seconds = seconds;
            self.progress_seconds = 0.0;
            true
        } else {
            false
        }
    }

    fn propagate(&mut self, event: &Message) {
        self.consumer.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.intermediate_buffer.fill(0.0);
        self.consumer
            .fill_buffer(self.intermediate_buffer.as_mut_slice());

        if self.progress_seconds >= self.duration_seconds {
            for (i, data) in buffer.iter_mut().enumerate() {
                *data += self.intermediate_buffer[i] * self.to_volume;
            }
            return;
        }

        let samples_to_fade = (((self.duration_seconds - self.progress_seconds)
            * consts::PLAYBACK_SAMPLE_RATE as f32) as usize)
            .min(buffer.len() / consts::CHANNEL_COUNT);
        let fade_gradient_per_sample = (self.to_volume - self.from_volume)
            / self.duration_seconds
            / (consts::PLAYBACK_SAMPLE_RATE as f32);
        let base_volume = self.from_volume
            + (self.progress_seconds / self.duration_seconds) * (self.to_volume - self.from_volume);

        for i in 0..samples_to_fade {
            let volume = base_volume + (i as f32) * fade_gradient_per_sample;
            buffer[2 * i] += self.intermediate_buffer[2 * i] * volume;
            buffer[2 * i + 1] += self.intermediate_buffer[2 * i + 1] * volume;
        }

        for (i, data) in buffer.iter_mut().enumerate().skip(2 * samples_to_fade) {
            *data += self.intermediate_buffer[i] * self.to_volume;
        }

        self.progress_seconds = (self.progress_seconds
            + ((buffer.len() / consts::CHANNEL_COUNT) as f32)
                / consts::PLAYBACK_SAMPLE_RATE as f32)
            .min(self.duration_seconds);
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User("Fader requires one child".to_owned()));
        }
        self.consumer = children[0].duplicate()?;
        Ok(())
    }
}
