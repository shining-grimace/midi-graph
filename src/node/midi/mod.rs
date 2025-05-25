pub mod cue;
pub mod util;

use crate::{
    Error, Event, EventTarget, Message, Node, consts,
    midi::{CueData, MidiEvent},
};
use midly::Smf;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::node::log;

pub struct MidiSourceBuilder {
    node_id: Option<u64>,
    midi_events: Vec<MidiEvent>,
    channel_sources: HashMap<usize, Box<dyn Node + Send + 'static>>,
    samples_per_tick: f64,
}

impl MidiSourceBuilder {
    /// Capture a non-static Smf, extracting MIDI event that contain text strings.
    /// Do not call to_static() on the Smf object before passing it in here!
    pub fn new(node_id: Option<u64>, smf: Smf) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let track_no = util::choose_track_index(&smf)?;
        if smf.tracks.len() > track_no + 1 {
            println!("WARNING: MIDI: Only the first track containing notes will be used");
        }
        let samples_per_tick = util::get_samples_per_tick(&smf)?;
        let midi_events = util::midi_events_from_midi(smf, track_no)?;
        Ok(Self {
            node_id,
            midi_events,
            channel_sources: HashMap::new(),
            samples_per_tick,
        })
    }

    /// Set up a builder using ready-to-go properties, but without any channel sources assigned
    fn new_empty_from_prepared_data(
        node_id: Option<u64>,
        midi_events: Vec<MidiEvent>,
        samples_per_tick: f64,
    ) -> Self {
        Self {
            node_id,
            midi_events,
            channel_sources: HashMap::new(),
            samples_per_tick,
        }
    }

    pub fn add_channel_source(
        mut self,
        channel: usize,
        source: Box<dyn Node + Send + 'static>,
    ) -> Self {
        self.channel_sources.insert(channel, source);
        self
    }

    pub fn build(self) -> Result<MidiSource, Error> {
        MidiSource::new(
            self.node_id,
            self.midi_events,
            self.channel_sources,
            self.samples_per_tick,
        )
    }
}

pub struct MidiSource {
    midi_events: Vec<MidiEvent>,
    node_id: u64,
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
        midi_events: Vec<MidiEvent>,
        channel_sources: HashMap<usize, Box<dyn Node + Send + 'static>>,
        samples_per_tick: f64,
    ) -> Result<Self, Error> {
        let mut sources: HashMap<usize, Box<dyn Node + Send + 'static>> = HashMap::new();

        for (channel, source) in channel_sources.into_iter() {
            if sources.insert(channel, source).is_some() {
                println!("WARNING: MIDI: Channel specified again will overwrite previous value");
            }
        }

        Ok(Self {
            midi_events,
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            queued_ideal_seek: None,
            channel_sources: sources,
            has_finished: false,
            samples_per_tick,
            next_event_index: 0,
            event_ticks_progress: 0,
        })
    }

    pub fn duplicate_without_sources(&self) -> MidiSourceBuilder {
        MidiSourceBuilder::new_empty_from_prepared_data(
            Some(self.node_id),
            self.midi_events.clone(),
            self.samples_per_tick,
        )
    }

    fn seek_to_anchor(&mut self, anchor: u32) {
        self.queued_ideal_seek = None;
        if let Some(index) = self.midi_events.iter().position(|c| match c.message {
            Message {
                data: Event::CueData(CueData::TargetMarker(a)),
                ..
            } => a == anchor,
            _ => false,
        }) {
            self.event_ticks_progress = 0;
            self.next_event_index = index + 1;
            let broadcast_cutoff = Message {
                target: EventTarget::Broadcast,
                data: Event::NoteOff { note: 0, vel: 1.0 },
            };
            for (_, source) in self.channel_sources.iter_mut() {
                source.on_event(&broadcast_cutoff);
            }
        };
    }

    fn on_internal_event_reached(&mut self, event: MidiEvent) {
        if let Event::CueData(cue) = &event.message.data {
            self.process_cue_event(cue);
            return;
        }
        let Some(source) = self.channel_sources.get_mut(&event.channel) else {
            return;
        };
        source.on_event(&event.message);
    }

    fn process_cue_event(&mut self, cue: &CueData) {
        match cue {
            CueData::TargetMarker(_) => { /* Marker, no action */ }
            CueData::GoodPointToSeekFrom => {
                if let Some(anchor) = self.queued_ideal_seek {
                    self.seek_to_anchor(anchor);
                    return;
                }
            }
            CueData::SeekNowToTarget(anchor) => {
                self.seek_to_anchor(*anchor);
            }
            CueData::SeekWhenIdeal(anchor) => {
                self.queued_ideal_seek = Some(*anchor);
            }
            CueData::ClearQueuedSeek => {
                self.queued_ideal_seek = None;
            }
        }
    }

    fn fill_all_channels(&mut self, buffer: &mut [f32]) {
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        loop {
            let reached_note_event = {
                let next_channel_event = &self.midi_events[self.next_event_index];
                let ticks_until_event = next_channel_event.delta_ticks - self.event_ticks_progress;
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
                if self.next_event_index >= self.midi_events.len() {
                    self.has_finished = true;
                    return;
                }

                next_channel_event
            };
            self.on_internal_event_reached(reached_note_event.clone());
        }
    }
}

impl Node for MidiSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        if !self.channel_sources.is_empty() {
            return Err(Error::User("MidiSource cannot be duplicated".to_owned()));
        }
        let source = Self::new(Some(self.node_id), self.midi_events.clone(), HashMap::new(), self.samples_per_tick)?;
        Ok(Box::new(source))
    }

    fn on_event(&mut self, event: &Message) {
        let was_consumed = if event.target.influences(self.node_id) {
            match &event.data {
                Event::CueData(cue) => {
                    self.process_cue_event(cue);
                    true
                }
                _ => false,
            }
        } else {
            false
        };
        if event.target.propagates_from(self.node_id, was_consumed) {
            for (_, source) in self.channel_sources.iter_mut() {
                source.on_event(event);
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if self.has_finished {
            return;
        }
        self.fill_all_channels(buffer);
    }

    fn replace_children(
        &mut self,
        _children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        Err(Error::User(
            "MidiSource does not support replacing its children".to_owned(),
        ))
    }
}
