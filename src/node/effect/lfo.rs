use core::f32;

use crate::{
    Balance, Error, Event, EventTarget, GraphNode, Message, Node, consts,
    effect::ModulationProperty,
};

pub struct Lfo {
    node_id: u64,
    property: Option<ModulationProperty>,
    consumer: GraphNode,
    frames_progress_in_step: isize,
    frames_per_step: isize,
    current_step: usize,
    cycle_steps: usize,
    low: f32,
    high: f32,
}

impl Lfo {
    pub fn new(node_id: Option<u64>, consumer: GraphNode) -> Result<Self, Error> {
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            property: None,
            consumer,
            frames_progress_in_step: 0,
            frames_per_step: 1,
            current_step: 0,
            cycle_steps: 0,
            low: 0.0,
            high: 1.0,
        })
    }

    fn send_step_event(&mut self) {
        let period_value = self.current_step as f32 / self.cycle_steps as f32;
        let value = self.low
            + (self.high - self.low) * ((period_value * 2.0 * f32::consts::PI).cos() * 0.5 + 0.5);
        let event = match self.property {
            Some(ModulationProperty::Volume) => Event::Volume(value),
            Some(ModulationProperty::Pan) => Event::SourceBalance(Balance::Pan(value)),
            Some(ModulationProperty::PitchMultiplier) => Event::PitchMultiplier(value),
            Some(ModulationProperty::MixBalance) => Event::MixerBalance(value),
            None => {
                return;
            }
        };
        self.consumer.on_event(&Message {
            target: EventTarget::FirstPossibleConsumer,
            data: event,
        });
    }

    fn send_off_event(&mut self) {
        let event = match self.property {
            Some(ModulationProperty::Volume) => Event::Volume(1.0),
            Some(ModulationProperty::Pan) => Event::SourceBalance(Balance::Both),
            Some(ModulationProperty::PitchMultiplier) => Event::PitchMultiplier(1.0),
            Some(ModulationProperty::MixBalance) => Event::MixerBalance(0.5),
            None => {
                return;
            }
        };
        self.consumer.on_event(&Message {
            target: EventTarget::FirstPossibleConsumer,
            data: event,
        });
    }
}

impl Node for Lfo {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumer = self.consumer.duplicate()?;
        let lfo = Self {
            node_id: self.node_id,
            property: self.property,
            consumer,
            frames_progress_in_step: 0,
            frames_per_step: self.frames_per_step,
            current_step: 0,
            cycle_steps: self.cycle_steps,
            low: 0.0,
            high: 1.0,
        };
        Ok(Box::new(lfo))
    }

    fn on_event(&mut self, event: &Message) {
        if event.target.influences(self.node_id) {
            match event.data {
                Event::Lfo {
                    property,
                    low,
                    high,
                    period_secs,
                    steps,
                } => {
                    let cycle_steps = if steps == 0 {
                        println!("WARNING: Cannot have zero steps for Lfo");
                        1
                    } else {
                        steps
                    };
                    let period_secs = if period_secs < f32::EPSILON {
                        println!(
                            "WARNING: Period for Lfo must be a positive, not-insignificant number"
                        );
                        1.0
                    } else {
                        period_secs
                    };
                    let frames_per_step: f32 =
                        consts::PLAYBACK_SAMPLE_RATE as f32 / (cycle_steps as f32 / period_secs);
                    self.property = Some(property);
                    self.low = low;
                    self.high = high;
                    self.frames_progress_in_step = 0;
                    self.frames_per_step = frames_per_step as isize;
                    self.current_step = 0;
                    self.cycle_steps = cycle_steps;
                }
                Event::EndModulation => {
                    self.send_off_event();
                    self.property = None;
                }
                _ => {}
            }
        }

        // Lfo does not consume any events, but listens to notes
        if event.target.propagates_from(self.node_id, false) {
            self.consumer.on_event(event);
        }
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
                if matches!(self.property, Some(ModulationProperty::PitchMultiplier)) {}
                self.send_step_event();
                if self.current_step >= self.cycle_steps {
                    self.current_step -= self.cycle_steps;
                }
            }
            frames_available -= frames_to_fill;
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User("Lfo requires one child".to_owned()));
        }
        self.consumer = children[0].duplicate()?;
        Ok(())
    }
}
