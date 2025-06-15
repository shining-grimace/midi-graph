use crate::{
    Balance, Error, Event, EventTarget, GraphNode, Message, Node,
    abstraction::{NodeConfigData, NodeRegistry, NodeConfig, defaults},
    consts,
    effect::ModulationProperty,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Transition {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub source: NodeConfigData,
}

impl NodeConfig for Transition {
    fn to_node(&self, registry: &NodeRegistry) -> Result<GraphNode, Error> {
        let source = self.source.0.to_node(registry)?;
        Ok(Box::new(TransitionNode::new(self.node_id, source)?))
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        Some(vec![
            self.source.clone()
        ])
    }

    fn duplicate(&self) -> Box<dyn NodeConfig> {
        Box::new(self.clone())
    }
}

pub struct TransitionNode {
    node_id: u64,
    property: Option<ModulationProperty>,
    consumer: GraphNode,
    frames_progress_in_step: isize,
    frames_per_step: isize,
    current_step: usize,
    total_steps: usize,
    from: f32,
    to: f32,
}

impl TransitionNode {
    pub fn new(node_id: Option<u64>, consumer: GraphNode) -> Result<Self, Error> {
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            property: None,
            consumer,
            frames_progress_in_step: 0,
            frames_per_step: 1,
            current_step: 0,
            total_steps: 0,
            from: 0.0,
            to: 1.0,
        })
    }

    fn send_event(&mut self) {
        let period_value = self.current_step as f32 / self.total_steps as f32;
        let value = self.from + (self.to - self.from) * period_value;
        let event = match self.property {
            Some(ModulationProperty::Volume) => Event::Volume(value),
            Some(ModulationProperty::Pan) => Event::SourceBalance(Balance::Pan(value)),
            Some(ModulationProperty::PitchMultiplier) => Event::PitchMultiplier(value),
            Some(ModulationProperty::MixBalance) => Event::MixerBalance(value),
            Some(ModulationProperty::TimeDilation) => Event::TimeDilation(value),
            None => {
                return;
            }
        };
        self.consumer.on_event(&Message {
            target: EventTarget::Broadcast,
            data: event,
        });
    }
}

impl Node for TransitionNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumer = self.consumer.duplicate()?;
        let transition = Self {
            node_id: self.node_id,
            property: self.property,
            consumer,
            frames_progress_in_step: 0,
            frames_per_step: self.frames_per_step,
            current_step: 0,
            total_steps: self.total_steps,
            from: 0.0,
            to: 1.0,
        };
        Ok(Box::new(transition))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::Transition {
                property,
                from,
                to,
                duration_secs,
                steps,
            } => {
                let total_steps = if steps == 0 {
                    println!("WARNING: Cannot have zero steps for TransitionEnvelope");
                    1
                } else {
                    steps
                };
                let duration_secs = if duration_secs < f32::EPSILON {
                    println!(
                        "WARNING: Duration for TransitionEnvelope must be a positive, not-insignificant number"
                    );
                    1.0
                } else {
                    duration_secs
                };
                let frames_per_step: f32 =
                    consts::PLAYBACK_SAMPLE_RATE as f32 / (steps as f32 / duration_secs);
                self.property = Some(property);
                self.from = from;
                self.to = to;
                self.frames_progress_in_step = 0;
                self.frames_per_step = frames_per_step as isize;
                self.current_step = 0;
                self.total_steps = total_steps;
            }
            Event::EndModulation => self.property = None,
            _ => {}
        }

        // TransitionEnvelope does not consume any events, but listens to notes
        false
    }

    fn propagate(&mut self, event: &Message) {
        self.consumer.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if self.property.is_none() {
            self.consumer.fill_buffer(buffer);
            return;
        }
        let buffer_size = buffer.len();
        let frames_in_buffer = buffer_size as isize / consts::CHANNEL_COUNT as isize;
        let mut frames_available = frames_in_buffer as isize;
        while frames_available > 0 {
            if self.property.is_none() {
                let buffer_index =
                    consts::CHANNEL_COUNT * (frames_in_buffer - frames_available) as usize;
                let intermediate_buffer = &mut buffer[buffer_index..];
                self.consumer.fill_buffer(intermediate_buffer);
                return;
            }
            let frames_left_in_step: isize = self.frames_per_step - self.frames_progress_in_step;
            let frames_to_fill = frames_left_in_step.min(frames_available as isize);
            let buffer_index =
                consts::CHANNEL_COUNT * (frames_in_buffer - frames_available) as usize;
            let buffer_end = buffer_index + consts::CHANNEL_COUNT * frames_to_fill as usize;
            let intermediate_slice = &mut buffer[buffer_index..buffer_end];
            self.consumer.fill_buffer(intermediate_slice);
            self.frames_progress_in_step += frames_to_fill as isize;
            if frames_to_fill == frames_left_in_step {
                self.frames_progress_in_step -= self.frames_per_step;
                self.current_step += 1;
                self.send_event();
                if self.current_step >= self.total_steps {
                    self.property = None;
                }
            }
            frames_available -= frames_to_fill;
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User(
                "TransitionEnvelope requires one child".to_owned(),
            ));
        }
        self.consumer = children[0].duplicate()?;
        Ok(())
    }
}
