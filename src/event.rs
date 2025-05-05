
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Message {
    pub target: EventTarget,
    pub data: Event
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EventTarget {

    /// Propagated everywhere without condition
    Broadcast,

    /// Handled in the first node that knows how to consume it
    FirstPossibleConsumer,

    /// Handled by a specific node
    SpecificNode(u64),
}

impl EventTarget {

    #[inline]
    pub fn influences(&self, node_id: u64) -> bool {
        match self {
            EventTarget::Broadcast => true,
            EventTarget::FirstPossibleConsumer => true,
            EventTarget::SpecificNode(id) => *id == node_id
        }
    }

    #[inline]
    pub fn propagates_from(&self, node_id: u64, was_consumed: bool) -> bool {
        match self {
            EventTarget::Broadcast => true,
            EventTarget::FirstPossibleConsumer => !was_consumed,
            EventTarget::SpecificNode(id) => *id != node_id
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    NoteOn { note: u8, vel: f32 },
    NoteOff {note: u8, vel: f32 },
    MixerBalance(f32),
    SourceBalance(Balance),
    Volume(f32),
    Fade { from: f32, to: f32, seconds: f32 },
    SeekWhenIdeal { to_anchor: Option<u32> },
    Unknown,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Balance {
    Both,
    Left,
    Right,
}

