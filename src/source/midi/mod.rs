pub mod cue;
pub mod util;

use crate::{
    consts, BroadcastControl, BufferConsumer, BufferConsumerNode, Cue, Error, FontSource,
    MidiDataSource, Node, NodeControlEvent, NodeEvent, NoteEvent, SoundFont, TimelineCue,
};
use midly::{MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind};
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::source::log;

#[derive(Debug)]
enum EventAction {
    ChannelNodeEvent {
        channel: usize,
        event: NodeEvent,
    },
    LoopCue {
        is_ideal_point: bool,
        seek_anchor: Option<u32>,
    },
}

pub struct MidiSourceBuilder {
    node_id: Option<u64>,
    smf: Smf<'static>,
    track_no: usize,
    timeline_cues: Vec<TimelineCue>,
    channel_fonts: HashMap<usize, SoundFont>,
}

impl MidiSourceBuilder {
    /// Capture a non-static Smf, extracting MIDI event that contain text strings.
    /// Do not call to_static() on the Smf object before passing it in here!
    pub fn new(node_id: Option<u64>, smf: Smf) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let track_no = util::choose_track_index(&smf)?;
        let timeline_cues = TimelineCue::from_smf(&smf, track_no)?;
        let static_smf = smf.to_static();
        if smf.tracks.len() > track_no + 1 {
            println!("WARNING: MIDI: Only the first track containing notes will be used");
        }
        Ok(Self {
            node_id,
            smf: static_smf,
            track_no,
            timeline_cues,
            channel_fonts: HashMap::new(),
        })
    }

    pub fn add_channel_font(mut self, channel: usize, font: SoundFont) -> Self {
        self.channel_fonts.insert(channel, font);
        self
    }

    pub fn build(self) -> Result<MidiSource, Error> {
        MidiSource::new(
            self.node_id,
            self.smf,
            self.track_no,
            self.timeline_cues,
            self.channel_fonts,
        )
    }
}

pub struct MidiSource {
    smf: RefCell<Smf<'static>>,
    node_id: u64,
    track_no: usize,
    timeline_cues: Vec<TimelineCue>,
    queued_ideal_seek: Option<u32>,
    channel_sources: HashMap<usize, Box<dyn Node + Send + 'static>>,
    has_finished: bool,
    samples_per_tick: f64,
    next_event_index: usize,
    event_ticks_progress: isize,
}

impl MidiSource {
    fn new(
        node_id: Option<u64>,
        smf: Smf<'static>,
        track_no: usize,
        timeline_cues: Vec<TimelineCue>,
        channel_fonts: HashMap<usize, SoundFont>,
    ) -> Result<Self, Error> {
        let samples_per_tick = util::get_samples_per_tick(&smf)?;
        let mut channel_sources: HashMap<usize, Box<dyn Node + Send + 'static>> = HashMap::new();

        for (channel, font) in channel_fonts.into_iter() {
            if channel_sources.insert(channel, Box::new(font)).is_some() {
                println!("WARNING: MIDI: Channel specified again will overwrite previous value");
            }
        }

        Ok(Self {
            smf: RefCell::new(smf),
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            track_no,
            timeline_cues,
            queued_ideal_seek: None,
            channel_sources,
            has_finished: false,
            samples_per_tick,
            next_event_index: 0,
            event_ticks_progress: 0,
        })
    }

    pub fn from_config(
        node_id: Option<u64>,
        source: &MidiDataSource,
        channels: &HashMap<usize, FontSource>,
    ) -> Result<Self, Error> {
        let mut midi_builder = match source {
            MidiDataSource::FilePath(file) => {
                crate::util::midi_builder_from_file(node_id, file.as_str())?
            }
        };
        for (channel, font_source) in channels.iter() {
            let soundfont = SoundFont::from_config(None, font_source)?;
            midi_builder = midi_builder.add_channel_font(*channel, soundfont);
        }
        midi_builder.build()
    }

    fn seek_to_anchor(&mut self, anchor: u32) {
        self.queued_ideal_seek = None;
        if let Some(index) = self.timeline_cues.iter().find_map(|c| match c {
            TimelineCue {
                event_index,
                cue: Cue::Anchor(a),
            } => match *a == anchor {
                true => Some(*event_index),
                false => None,
            },
            _ => None,
        }) {
            self.event_ticks_progress = 0;
            self.next_event_index = index + 1;
            let broadcast_cutoff = NodeEvent::Broadcast(BroadcastControl::NotesOff);
            for (_, source) in self.channel_sources.iter_mut() {
                source.on_event(&broadcast_cutoff);
            }
        };
    }

    fn on_event_reached(&mut self, event: &Option<EventAction>) {
        match event {
            None => {}
            Some(EventAction::ChannelNodeEvent { channel, event }) => {
                let Some(source) = self.channel_sources.get_mut(channel) else {
                    return;
                };
                source.on_event(event);
            }
            Some(EventAction::LoopCue {
                is_ideal_point,
                seek_anchor,
            }) => {
                if *is_ideal_point {
                    if let Some(anchor) = self.queued_ideal_seek {
                        self.seek_to_anchor(anchor);
                        return;
                    }
                }
                if let Some(anchor) = seek_anchor {
                    self.seek_to_anchor(*anchor);
                }
            }
        }
    }

    fn note_event_from_midi_event(
        &self,
        at_track_index: usize,
        event: &TrackEvent,
    ) -> Option<EventAction> {
        match event.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } => Some(EventAction::ChannelNodeEvent {
                channel: u8::from(channel) as usize,
                event: NodeEvent::Note {
                    note: u8::from(key),
                    event: NoteEvent::NoteOn {
                        vel: u8::from(vel) as f32 / 127.0,
                    },
                },
            }),
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff { key, vel },
            } => Some(EventAction::ChannelNodeEvent {
                channel: u8::from(channel) as usize,
                event: NodeEvent::Note {
                    note: u8::from(key),
                    event: NoteEvent::NoteOff {
                        vel: u8::from(vel) as f32 / 127.0,
                    },
                },
            }),
            TrackEventKind::Meta(MetaMessage::CuePoint(_)) => {
                let is_ideal_point = self.timeline_cues.iter().any(|c| match c {
                    TimelineCue {
                        event_index,
                        cue: Cue::IdealSeekPoint,
                    } => *event_index == at_track_index,
                    _ => false,
                });
                let seek_anchor = self.timeline_cues.iter().find_map(|c| match c {
                    TimelineCue {
                        event_index,
                        cue: Cue::Seek(anchor),
                    } => match *event_index == at_track_index {
                        true => Some(*anchor),
                        false => None,
                    },
                    _ => None,
                });
                match is_ideal_point || seek_anchor.is_some() {
                    true => Some(EventAction::LoopCue {
                        is_ideal_point,
                        seek_anchor,
                    }),
                    false => None,
                }
            }
            _ => None,
        }
    }

    fn fill_all_channels(&mut self, buffer: &mut [f32]) {
        if self.has_finished {
            return;
        }
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        loop {
            let reached_note_event = {
                let smf = self.smf.borrow();
                let track_data = &smf.tracks[self.track_no];
                let next_event = &track_data[self.next_event_index];
                let event_ticks_delta = u32::from(next_event.delta) as isize;
                let ticks_until_event = event_ticks_delta - self.event_ticks_progress;
                let samples_until_event =
                    (ticks_until_event as f64 * self.samples_per_tick) as usize;
                let samples_available_per_channel = buffer.len() / consts::CHANNEL_COUNT;

                {
                    if samples_until_event > samples_available_per_channel {
                        for (_, source) in self.channel_sources.iter_mut() {
                            source.fill_buffer(buffer);
                        }
                        self.event_ticks_progress +=
                            (samples_available_per_channel as f64 / self.samples_per_tick) as isize;
                        return;
                    }

                    let buffer_samples_to_fill = samples_until_event * consts::CHANNEL_COUNT;
                    for (_, source) in self.channel_sources.iter_mut() {
                        source.fill_buffer(&mut buffer[0..buffer_samples_to_fill]);
                    }
                }

                self.event_ticks_progress = 0;
                self.next_event_index += 1;
                if self.next_event_index >= track_data.len() {
                    self.has_finished = true;
                    return;
                }

                self.note_event_from_midi_event(self.next_event_index - 1, next_event)
            };
            self.on_event_reached(&reached_note_event);
        }
    }
}

impl BufferConsumerNode for MidiSource {}

impl Node for MidiSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, event: &NodeEvent) {
        if let NodeEvent::NodeControl {
            node_id,
            event: NodeControlEvent::SeekWhenIdeal { to_anchor },
        } = event
        {
            if *node_id == self.node_id {
                self.queued_ideal_seek = *to_anchor;
                return;
            }
        }
        for (_, source) in self.channel_sources.iter_mut() {
            source.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.fill_all_channels(buffer);
    }
}

impl BufferConsumer for MidiSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        Err(Error::User("MidiSource cannot be duplicated".to_owned()))
    }
}
