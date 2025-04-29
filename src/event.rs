
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum NodeEvent {
    Broadcast(BroadcastControl),
    Note {
        note: u8,
        event: NoteEvent,
    },
    NodeControl {
        node_id: u64,
        event: NodeControlEvent,
    },
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum BroadcastControl {
    NotesOff,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum NoteEvent {
    NoteOn { vel: f32 },
    NoteOff { vel: f32 },
}

#[derive(Clone, Debug)]
pub enum NodeControlEvent {
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

