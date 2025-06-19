pub mod cue;
pub mod util;

use crate::{
    AssetLoader, Error, Event, EventTarget, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
    consts,
    midi::{CueData, MidiEvent},
    util as file_util,
};
use midly::Smf;
use serde::Deserialize;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::node::log;

#[derive(Deserialize, Clone)]
pub enum MidiDataSource {
    FilePath(String),
}

#[derive(Deserialize, Clone)]
pub struct Midi {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub source: MidiDataSource,
    pub channels: HashMap<usize, NodeConfigData>,
}

impl NodeConfig for Midi {
    fn to_node(&self, asset_loader: &Box<dyn AssetLoader>) -> Result<GraphNode, Error> {
        let mut midi_builder = match &self.source {
            MidiDataSource::FilePath(file) => {
                let bytes = asset_loader.load_asset_data(&file)?;
                file_util::midi_builder_from_bytes(self.node_id, &bytes)?
            }
        };
        for (channel, source) in self.channels.iter() {
            let source = source.0.to_node(asset_loader)?;
            midi_builder = midi_builder.add_channel_source(*channel, source);
        }
        let source = midi_builder.build()?;
        let source: GraphNode = Box::new(source);
        Ok(source)
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        Some(
            self.channels
                .iter()
                .map(|(_, config)| config.clone())
                .collect(),
        )
    }

    fn asset_source(&self) -> Option<&str> {
        match &self.source {
            MidiDataSource::FilePath(path) => Some(path),
        }
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct MidiNodeBuilder {
    node_id: Option<u64>,
    midi_events: Vec<MidiEvent>,
    channel_sources: HashMap<usize, GraphNode>,
    samples_per_tick: f64,
}

impl MidiNodeBuilder {
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

    pub fn add_channel_source(mut self, channel: usize, source: GraphNode) -> Self {
        self.channel_sources.insert(channel, source);
        self
    }

    pub fn build(self) -> Result<MidiNode, Error> {
        MidiNode::new(
            self.node_id,
            self.midi_events,
            self.channel_sources,
            self.samples_per_tick,
        )
    }
}

pub struct MidiNode {
    midi_events: Vec<MidiEvent>,
    node_id: u64,
    queued_ideal_seek: Option<u32>,
    channel_sources: HashMap<usize, GraphNode>,
    has_finished: bool,
    samples_per_tick: f64,
    next_event_index: usize,
    event_ticks_progress: isize,
    time_dilation: f32,
}

impl MidiNode {
    fn new(
        node_id: Option<u64>,
        midi_events: Vec<MidiEvent>,
        channel_sources: HashMap<usize, GraphNode>,
        samples_per_tick: f64,
    ) -> Result<Self, Error> {
        let mut sources: HashMap<usize, GraphNode> = HashMap::new();

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
            time_dilation: 1.0,
        })
    }

    pub fn duplicate_without_sources(&self) -> MidiNodeBuilder {
        MidiNodeBuilder::new_empty_from_prepared_data(
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
                let samples_until_event = (ticks_until_event as f64
                    * (self.samples_per_tick / self.time_dilation as f64))
                    as usize;
                let samples_available_per_channel = buffer.len() / consts::CHANNEL_COUNT;

                {
                    if samples_until_event > samples_available_per_channel {
                        for (_, source) in self.channel_sources.iter_mut() {
                            source.fill_buffer(buffer);
                        }
                        self.event_ticks_progress += (samples_available_per_channel as f64
                            / (self.samples_per_tick / self.time_dilation as f64))
                            as isize;
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

impl Node for MidiNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        if !self.channel_sources.is_empty() {
            return Err(Error::User("MidiSource cannot be duplicated".to_owned()));
        }
        let source = Self::new(
            Some(self.node_id),
            self.midi_events.clone(),
            HashMap::new(),
            self.samples_per_tick,
        )?;
        Ok(Box::new(source))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        println!("Received event {:?}", event);
        match &event.data {
            Event::CueData(cue) => {
                self.process_cue_event(cue);
                true
            }
            Event::TimeDilation(value) => {
                self.time_dilation = *value;
                true
            }
            _ => false,
        }
    }

    fn propagate(&mut self, event: &Message) {
        for (_, source) in self.channel_sources.iter_mut() {
            source.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if self.has_finished {
            return;
        }
        self.fill_all_channels(buffer);
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if !self.channel_sources.is_empty() {
            return Err(Error::User(
                "MidiSource does not support replacing its children".to_owned(),
            ));
        }
        println!(
            "MIDI Graph: Assigning channel sources to MIDI source; assuming sequential channel numbers starting at 1."
        );
        println!(
            "This is a current limitation. Please check your source file channel numbers if needed."
        );
        self.channel_sources = children
            .iter()
            .enumerate()
            .map(|(index, source)| source.duplicate().map(|copy| (index + 1, copy)))
            .collect::<Result<HashMap<usize, GraphNode>, Error>>()?;
        Ok(())
    }
}
