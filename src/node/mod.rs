pub mod effect;
pub mod font;
pub mod generator;
pub mod group;
pub mod midi;
pub mod util;

#[cfg(debug_assertions)]
pub mod log;

use crate::{Error, EventTarget, GraphNode, Loop, Message, RangeSource};
use std::sync::atomic::{AtomicU64, Ordering};

const START_GENERATED_NODE_IDS: u64 = 0x10000;
static NEXT_ID: AtomicU64 = AtomicU64::new(START_GENERATED_NODE_IDS);

pub trait Node {
    fn get_node_id(&self) -> u64;
    fn set_node_id(&mut self, node_id: u64);
    fn duplicate(&self) -> Result<GraphNode, Error>;
    fn try_consume_event(&mut self, event: &Message) -> bool;
    fn propagate(&mut self, event: &Message);
    fn fill_buffer(&mut self, buffer: &mut [f32]);
    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error>;

    fn on_event(&mut self, event: &Message) {
        let node_id = self.get_node_id();
        let was_consumed = if event.target.influences(node_id) {
            self.try_consume_event(event)
        } else {
            false
        };
        let broadcast_propagate = match event.target {
            EventTarget::SpecificNode(node_id) => node_id == self.get_node_id(),
            _ => false,
        };
        let propagation_event = match broadcast_propagate {
            true => &Message {
                target: EventTarget::Broadcast,
                data: event.data.clone(),
            },
            false => event,
        };
        if propagation_event
            .target
            .propagates_from(node_id, was_consumed)
        {
            self.propagate(propagation_event);
        }
    }

    fn new_node_id() -> u64
    where
        Self: Sized,
    {
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct NoteRange {
    pub lower_inclusive: u8,
    pub upper_inclusive: u8,
}

impl NoteRange {
    pub fn new_inclusive_range(lower: u8, upper: u8) -> Self {
        Self {
            lower_inclusive: lower,
            upper_inclusive: upper,
        }
    }

    pub fn new_full_range() -> Self {
        Self {
            lower_inclusive: 0,
            upper_inclusive: 255,
        }
    }

    pub fn from_config(config: &RangeSource) -> Self {
        Self {
            lower_inclusive: config.lower,
            upper_inclusive: config.upper,
        }
    }

    pub fn contains(&self, note: u8) -> bool {
        self.lower_inclusive <= note && self.upper_inclusive >= note
    }
}

pub struct LoopRange {
    pub start_frame: usize,
    pub end_frame: usize,
}

impl LoopRange {
    pub fn new_frame_range(start_frame: usize, end_frame: usize) -> Self {
        Self {
            start_frame,
            end_frame,
        }
    }

    pub fn from_config(config: &Loop) -> Self {
        Self {
            start_frame: config.start,
            end_frame: config.end,
        }
    }
}
