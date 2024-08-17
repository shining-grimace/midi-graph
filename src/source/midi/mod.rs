pub mod chunk;
pub mod track;
pub mod util;

use crate::{BufferConsumer, Error, MidiChunkSource, NoteEvent, SoundFont};
use midly::Smf;

#[cfg(debug_assertions)]
use crate::source::log;

pub struct MidiSource<'a> {
    source: Box<MidiChunkSource<'a>>,
    has_finished: bool,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, track_fonts: Vec<SoundFont>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let source = MidiChunkSource::new(smf, track_fonts)?;

        Ok(Self {
            source: Box::new(source),
            has_finished: false,
        })
    }
}

impl<'a> BufferConsumer for MidiSource<'a> {
    fn set_note(&mut self, event: NoteEvent) {
        self.has_finished = match event {
            NoteEvent::NoteOn(_) => true,
            NoteEvent::NoteOff(_) => false,
        };
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.source.fill_buffer(buffer);
    }
}
