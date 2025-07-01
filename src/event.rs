use crate::{effect::ModulationProperty, midi::CueData};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Message {
    pub target: EventTarget,
    pub data: Event,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EventTarget {
    /// Handled by all nodes reached.
    /// Propagates through graph branches until consumed.
    Broadcast,

    /// Handled by a specific node.
    /// Still propagates through graph branches until consumed.
    SpecificNode(u64),
}

impl EventTarget {
    #[inline]
    pub fn influences(&self, node_id: u64) -> bool {
        match self {
            EventTarget::Broadcast => true,
            EventTarget::SpecificNode(id) => *id == node_id,
        }
    }

    #[inline]
    pub fn propagates_from(&self, node_id: u64, was_consumed: bool) -> bool {
        match self {
            EventTarget::Broadcast => !was_consumed,
            EventTarget::SpecificNode(id) => *id != node_id,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    CueData(CueData),
    LoopCue {
        is_ideal_point: bool,
        seek_anchor: Option<u32>,
    },
    NoteOn {
        note: u8,
        vel: f32,
    },
    NoteOff {
        note: u8,
        vel: f32,
    },
    MixerBalance(f32),
    SourceBalance(Balance),
    Volume(f32),
    PitchMultiplier(f32),
    TimeDilation(f32),
    FilterFrequencyShift(f32),
    Fade {
        from: f32,
        to: f32,
        seconds: f32,
    },
    Transition {
        property: ModulationProperty,
        from: f32,
        to: f32,
        duration_secs: f32,
        steps: usize,
    },
    Lfo {
        property: ModulationProperty,
        low: f32,
        high: f32,
        period_secs: f32,
        steps: usize,
    },
    Filter {
        filter: IirFilter,
        cutoff_frequency: f32,
    },
    EndModulation,
    Unknown,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Balance {
    Both,
    Left,
    Right,
    Pan(f32),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum IirFilter {
    SinglePoleLowPassApprox,
    SinglePoleLowPass,
    LowPass,
    HighPass,
    BandPass,
    Notch,
    AllPass,
    LowShelf { db_gain: f32 },
    HighShelf { db_gain: f32 },
    PeakingEQ { db_gain: f32 },
}
