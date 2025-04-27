use crate::Error;
use midly::{MetaMessage, Smf, TrackEventKind};

#[derive(Copy, Clone, Debug)]
pub struct TimelineCue {
    pub event_index: usize,
    pub cue: Cue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Cue {
    Anchor(u32),
    IdealSeekPoint,
    Seek(u32),
}

impl TimelineCue {
    pub fn from_smf(smf: &Smf, track_index: usize) -> Result<Vec<Self>, Error> {
        let mut cues = vec![];
        for (event_index, event) in smf.tracks[track_index].iter().enumerate() {
            if let TrackEventKind::Meta(MetaMessage::CuePoint(label)) = event.kind {
                let string = std::str::from_utf8(label).map_err(|_| {
                    Error::Internal("Cannot parse event label in MIDI data".to_owned())
                })?;
                let length = string.chars().count();
                let mut index = 0;
                while index < length {
                    match string.chars().nth(index) {
                        Some('#') => {
                            let start_index = index + 1;
                            let mut end_index = start_index;
                            while end_index < length {
                                if let Some(ch) = string.chars().nth(end_index) {
                                    if ch.is_numeric() {
                                        end_index += 1;
                                        continue;
                                    }
                                }
                                break;
                            }
                            if end_index == start_index {
                                println!("WARNING: MIDI: Cannot parse anchor in label {}", string);
                            } else {
                                let anchor_index =
                                    &string[start_index..end_index].parse().map_err(|_| {
                                        Error::User(format!(
                                            "Failed parsing anchor index in label {}",
                                            string
                                        ))
                                    })?;
                                cues.push(TimelineCue {
                                    event_index,
                                    cue: Cue::Anchor(*anchor_index),
                                });
                            }
                            index = end_index;
                        }
                        Some('>') => {
                            let start_index = index + 1;
                            let mut end_index = start_index;
                            while end_index < length {
                                if let Some(ch) = string.chars().nth(end_index) {
                                    if ch.is_numeric() {
                                        end_index += 1;
                                        continue;
                                    }
                                }
                                break;
                            }
                            if end_index == start_index {
                                println!("WARNING: MIDI: Cannot parse seek label");
                            } else {
                                let anchor_index =
                                    &string[start_index..end_index].parse().map_err(|_| {
                                        Error::User(format!(
                                            "Failed parsing seek index in label {}",
                                            string
                                        ))
                                    })?;
                                cues.push(TimelineCue {
                                    event_index,
                                    cue: Cue::Seek(*anchor_index),
                                });
                            }
                            index = end_index;
                        }
                        Some('?') => {
                            cues.push(TimelineCue {
                                event_index,
                                cue: Cue::IdealSeekPoint,
                            });
                            index += 1;
                        }
                        _ => {
                            println!("WARNING: MIDI: Unknown data in cue point label");
                            break;
                        }
                    }
                }
            }
        }
        Ok(cues)
    }
}
