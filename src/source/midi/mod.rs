pub mod track;
pub mod util;

use crate::{
    util::midi_builder_from_file, BufferConsumer, BufferConsumerNode, Config, Error,
    MidiDataSource, MidiTrackSource, Node, NodeEvent, SoundFont,
};
use midly::Smf;
use std::{collections::HashMap, sync::Arc};

#[cfg(debug_assertions)]
use crate::source::log;

struct MidiTimeEvents;

impl MidiTimeEvents {
    fn from_smf<'a>(_smf: &Smf<'a>) -> Self {
        Self
    }
}

pub struct MidiSourceBuilder {
    smf: Smf<'static>,
    time_events: MidiTimeEvents,
    channel_fonts: HashMap<usize, SoundFont>,
}

impl MidiSourceBuilder {
    /// Capture a non-static Smf, extracting MIDI event that contain text strings.
    /// Do not call to_static() on the Smf object before passing it in here!
    pub fn new<'a>(smf: Smf<'a>) -> Self {
        let time_events = MidiTimeEvents::from_smf(&smf);
        let static_smf = smf.to_static();
        Self {
            smf: static_smf,
            time_events,
            channel_fonts: HashMap::new(),
        }
    }

    pub fn add_channel_font(mut self, channel: usize, font: SoundFont) -> Self {
        self.channel_fonts.insert(channel, font);
        self
    }

    pub fn build(self) -> Result<MidiSource, Error> {
        MidiSource::new(self.smf, self.channel_fonts)
    }
}

pub struct MidiSource {
    node_id: u64,
    channel_sources: HashMap<usize, Box<MidiTrackSource>>,
}

impl MidiSource {
    fn new(smf: Smf<'static>, channel_fonts: HashMap<usize, SoundFont>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let samples_per_tick = util::get_samples_per_tick(&smf)?;
        let track_index = util::choose_track_index(&smf)?;
        if smf.tracks.len() > track_index + 1 {
            println!("WARNING: MIDI: Only the first track containing notes will be used");
        }
        let mut channel_sources = HashMap::new();
        let smf_arc = Arc::new(smf);

        for (channel, font) in channel_fonts.into_iter() {
            let source = MidiTrackSource::new(
                Arc::clone(&smf_arc),
                track_index,
                channel,
                samples_per_tick,
                Box::new(font),
            );
            channel_sources
                .insert(channel, Box::new(source))
                .and_then(|_| {
                    println!(
                        "WARNING: MIDI: Channel specified again will overwrite previous value"
                    );
                    Some(())
                });
        }

        Ok(Self {
            node_id: <Self as Node>::new_node_id(),
            channel_sources,
        })
    }

    pub fn from_config(config: Config) -> Result<Self, Error> {
        let mut midi_builder = match config.midi {
            MidiDataSource::FilePath(file) => midi_builder_from_file(file.as_str())?,
        };
        for (channel, font_source) in config.channels.iter() {
            let soundfont = SoundFont::from_config(font_source)?;
            midi_builder = midi_builder.add_channel_font(*channel, soundfont);
        }
        midi_builder.build()
    }
}

impl BufferConsumerNode for MidiSource {}

impl Node for MidiSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, _event: &NodeEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for (_, source) in self.channel_sources.iter_mut() {
            source.fill_buffer(buffer);
        }
    }
}

impl BufferConsumer for MidiSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        Err(Error::User("MidiSource cannot be duplicated".to_owned()))
    }
}
