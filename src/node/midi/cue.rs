use crate::Error;

#[derive(Clone, Debug)]
pub enum CueData {
    TargetMarker(u32),
    GoodPointToSeekFrom,
    SeekNowToTarget(u32),
    SeekWhenIdeal(u32),
    ClearQueuedSeek,
}

impl CueData {
    pub fn from_label(label: &[u8]) -> Result<Vec<Self>, Error> {
        let string = std::str::from_utf8(label)
            .map_err(|_| Error::Internal("Cannot parse event label in MIDI data".to_owned()))?;
        let mut cues: Vec<Self> = vec![];
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
                        cues.push(Self::TargetMarker(*anchor_index));
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
                        cues.push(Self::SeekNowToTarget(*anchor_index));
                    }
                    index = end_index;
                }
                Some('?') => {
                    cues.push(Self::GoodPointToSeekFrom);
                    index += 1;
                }
                _ => {
                    println!("WARNING: MIDI: Unknown data in cue point label");
                    break;
                }
            }
        }
        Ok(cues)
    }
}
