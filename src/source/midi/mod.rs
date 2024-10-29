pub mod cue;
pub mod util;

use crate::{
    consts, BufferConsumer, BufferConsumerNode, Config, Error, MidiDataSource, Node, NodeEvent,
    NoteEvent, SoundFont, TimelineCue,
};
use midly::{MidiMessage, Smf, TrackEvent, TrackEventKind};
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::source::log;

struct NodeEventOnChannel {
    channel: usize,
    event: NodeEvent,
}

pub struct MidiSourceBuilder {
    smf: Smf<'static>,
    track_no: usize,
    timeline_cues: Vec<(u64, TimelineCue)>,
    channel_fonts: HashMap<usize, SoundFont>,
}

impl MidiSourceBuilder {
    /// Capture a non-static Smf, extracting MIDI event that contain text strings.
    /// Do not call to_static() on the Smf object before passing it in here!
    pub fn new<'a>(smf: Smf<'a>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let track_no = util::choose_track_index(&smf)?;
        let timeline_cues = TimelineCue::from_smf(&smf, track_no)?;
        let static_smf = smf.to_static();
        if smf.tracks.len() > track_no + 1 {
            println!("WARNING: MIDI: Only the first track containing notes will be used");
        }
        Ok(Self {
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
    timeline_cues: Vec<(u64, TimelineCue)>,
    channel_sources: HashMap<usize, Box<dyn Node + Send + 'static>>,
    has_finished: bool,
    samples_per_tick: f64,
    next_event_index: usize,
    event_ticks_progress: isize,
}

impl MidiSource {
    fn new(
        smf: Smf<'static>,
        track_no: usize,
        timeline_cues: Vec<(u64, TimelineCue)>,
        channel_fonts: HashMap<usize, SoundFont>,
    ) -> Result<Self, Error> {
        let samples_per_tick = util::get_samples_per_tick(&smf)?;
        let mut channel_sources: HashMap<usize, Box<dyn Node + Send + 'static>> = HashMap::new();

        for (channel, font) in channel_fonts.into_iter() {
            channel_sources
                .insert(channel, Box::new(font))
                .and_then(|_| {
                    println!(
                        "WARNING: MIDI: Channel specified again will overwrite previous value"
                    );
                    Some(())
                });
        }

        Ok(Self {
            smf: RefCell::new(smf),
            node_id: <Self as Node>::new_node_id(),
            track_no,
            timeline_cues,
            channel_sources,
            has_finished: false,
            samples_per_tick,
            next_event_index: 0,
            event_ticks_progress: 0,
        })
    }

    pub fn from_config(config: Config) -> Result<Self, Error> {
        let mut midi_builder = match config.midi {
            MidiDataSource::FilePath(file) => crate::util::midi_builder_from_file(file.as_str())?,
        };
        for (channel, font_source) in config.channels.iter() {
            let soundfont = SoundFont::from_config(font_source)?;
            midi_builder = midi_builder.add_channel_font(*channel, soundfont);
        }
        midi_builder.build()
    }

    fn on_event_reached(&mut self, event: &Option<NodeEventOnChannel>) {
        match event {
            None => {}
            Some(e) => {
                let Some(source) = self.channel_sources.get_mut(&e.channel) else {
                    return;
                };
                source.on_event(&e.event);
            }
        }
    }

    fn note_event_from_midi_event(event: &TrackEvent) -> Option<NodeEventOnChannel> {
        match event.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } => Some(NodeEventOnChannel {
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
            } => Some(NodeEventOnChannel {
                channel: u8::from(channel) as usize,
                event: NodeEvent::Note {
                    note: u8::from(key),
                    event: NoteEvent::NoteOff {
                        vel: u8::from(vel) as f32 / 127.0,
                    },
                },
            }),
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

                Self::note_event_from_midi_event(next_event)
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
