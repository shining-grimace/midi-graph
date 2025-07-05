use crate::{Error, Event, EventTarget, Message, midi::CueData};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

pub fn midi_events_from_midi(smf: Smf, track_index: usize) -> Result<Vec<MidiEvent>, Error> {
    let mut midi_events: Vec<MidiEvent> = vec![];
    let track = smf
        .tracks
        .get(track_index)
        .ok_or_else(|| Error::User(format!("ERROR: MIDI: No track no. {}", track_index)))?;
    for event in track {
        let event_delta = u32::from(event.delta) as isize;
        match event.kind {
            // Special case for cue labels since they encode multiple events
            TrackEventKind::Meta(MetaMessage::CuePoint(label)) => {
                let cue_data = CueData::from_label(label)?;
                let events = MidiEvent::from_cue_data(event_delta, cue_data);
                for cue_event in events.into_iter() {
                    midi_events.push(cue_event);
                }
            }
            _ => {
                if let Some(graph_event) = MidiEvent::from_midi_event(event_delta, &event.kind) {
                    midi_events.push(graph_event);
                }
            }
        }
    }
    Ok(midi_events)
}

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub delta_ticks: isize,
    pub channel: usize,
    pub message: Message,
}

impl MidiEvent {
    pub fn from_midi_event(event_delta: isize, event_kind: &TrackEventKind) -> Option<Self> {
        match event_kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } => Some(MidiEvent {
                delta_ticks: event_delta,
                channel: u8::from(*channel) as usize,
                message: Message {
                    target: EventTarget::Broadcast,
                    data: Event::NoteOn {
                        note: u8::from(*key),
                        vel: u8::from(*vel) as f32 / 127.0,
                    },
                },
            }),
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff { key, vel },
            } => Some(MidiEvent {
                delta_ticks: event_delta,
                channel: u8::from(*channel) as usize,
                message: Message {
                    target: EventTarget::Broadcast,
                    data: Event::NoteOff {
                        note: u8::from(*key),
                        vel: u8::from(*vel) as f32 / 127.0,
                    },
                },
            }),
            _ => None,
        }
    }

    pub fn from_cue_data(event_delta: isize, cue_data: Vec<CueData>) -> Vec<Self> {
        let mut event_delta = event_delta;
        let mut midi_events = vec![];
        for cue in cue_data.into_iter() {
            midi_events.push(MidiEvent {
                delta_ticks: event_delta,
                channel: 0, // Ignored for cue data
                message: Message {
                    target: EventTarget::Broadcast,
                    data: Event::CueData(cue),
                },
            });
            event_delta = 0;
        }
        midi_events
    }
}
