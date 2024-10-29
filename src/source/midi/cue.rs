use crate::Error;
use midly::{MetaMessage, Smf, TrackEventKind};

pub enum TimelineCue {
    Anchor(u32),
    IdealSeekPoint,
    Seek(u32),
}

impl TimelineCue {
    pub fn from_smf<'a>(smf: &Smf<'a>, track_index: usize) -> Result<Vec<(u64, Self)>, Error> {
        let mut total_delta: u64 = 0;
        let mut cues = vec![];
        for event in smf.tracks[track_index].iter() {
            total_delta += u32::from(event.delta) as u64;
            match event.kind {
                TrackEventKind::Meta(MetaMessage::CuePoint(label)) => {
                    let string = std::str::from_utf8(label).or_else(|_| {
                        Err(Error::User(
                            "ERROR: MIDI: Cannot parse event label".to_owned(),
                        ))
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
                                    println!("WARNING: MIDI: Cannot parse anchor label");
                                } else {
                                    let anchor_index =
                                        &string[start_index..end_index].parse().or_else(|_| {
                                            Err(Error::User(
                                                "Failed parsing anchor index".to_owned(),
                                            ))
                                        })?;
                                    cues.push((total_delta, TimelineCue::Anchor(*anchor_index)));
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
                                        &string[start_index..end_index].parse().or_else(|_| {
                                            Err(Error::User("Failed parsing seek index".to_owned()))
                                        })?;
                                    cues.push((total_delta, TimelineCue::Seek(*anchor_index)));
                                }
                                index = end_index;
                            }
                            Some('?') => {
                                cues.push((total_delta, TimelineCue::IdealSeekPoint));
                                index += 1;
                            }
                            _ => {
                                println!("WARNING: MIDI: Unknown data in cue point label");
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(cues)
    }
}
