pub mod chunk;
pub mod track;
pub mod util;

use crate::{BufferConsumer, Error, MidiChunkSource, NoteEvent, NoteKind, SoundFont};
use midly::Smf;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::source::log;

pub struct MidiSourceBuilder<'a> {
    smf: Smf<'a>,
    channel_fonts: HashMap<usize, SoundFont>,
}

impl<'a> MidiSourceBuilder<'a> {
    pub fn new(smf: Smf<'a>) -> Self {
        Self {
            smf,
            channel_fonts: HashMap::new(),
        }
    }

    pub fn add_channel_font(mut self, channel: usize, font: SoundFont) -> Self {
        self.channel_fonts.insert(channel, font);
        self
    }

    pub fn build(self) -> Result<MidiSource<'a>, Error> {
        MidiSource::new(self.smf, self.channel_fonts)
    }
}

pub struct MidiSource<'a> {
    source: Box<MidiChunkSource<'a>>,
    has_finished: bool,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, channel_fonts: HashMap<usize, SoundFont>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let source = MidiChunkSource::new(smf, channel_fonts)?;

        Ok(Self {
            source: Box::new(source),
            has_finished: false,
        })
    }
}

impl<'a> BufferConsumer for MidiSource<'a> {
    fn set_note(&mut self, event: NoteEvent) {
        self.has_finished = match event.kind {
            NoteKind::NoteOn(_) => true,
            NoteKind::NoteOff(_) => false,
        };
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.source.fill_buffer(buffer);
    }
}
