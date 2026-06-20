use crate::{consts, effect::ModulationProperty, midi::CueData};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Message {
    pub target: EventTarget,
    pub data: Event,
    pub timing: EventTiming,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            target: EventTarget::Broadcast,
            data: Event::Unknown,
            timing: EventTiming::Imprecise,
        }
    }
}

impl Message {
    pub fn broadcast(event: Event) -> Self {
        Self {
            data: event,
            ..Self::default()
        }
    }
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

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EventTiming {
    /// The event should be handled at the next possible time, which will be
    /// on the next loop in the audio thread
    Imprecise,
    /// The event should be handled at a precise future time (or as soon as
    /// possible if the specified time is already passed)
    AtAbsoluteFrame(u64),
}

impl EventTiming {
    pub fn after_seconds(absolute_frame: u64, seconds: f32) -> Self {
        Self::AtAbsoluteFrame(
            absolute_frame + (seconds * consts::PLAYBACK_SAMPLE_RATE as f32) as u64,
        )
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    StateSnapshot(Value),
    CueData(CueData),
    LoopCue {
        is_ideal_point: bool,
        seek_anchor: Option<u32>,
    },
    MidiPlayback(MidiPlaybackState),
    NoteOn {
        note: u8,
        vel: f32,
    },
    NoteOff {
        note: u8,
        vel: f32,
    },
    AllNotesOff,
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
    Wavetable(Vec<f32>),
    Unknown,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MidiPlaybackState {
    Playing,
    Paused,
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
