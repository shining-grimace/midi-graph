pub mod chunk;
pub mod track;
pub mod util;

use crate::{AudioSource, Error, MidiChunkSource};
use midly::Smf;

#[cfg(debug_assertions)]
use crate::source::log;

pub struct MidiSource<'a> {
    source: Box<MidiChunkSource<'a>>,
    has_finished: bool,
}

impl<'a> MidiSource<'a> {
    pub fn new(
        smf: Smf<'a>,
        note_source_spawner: fn() -> Box<dyn AudioSource + Send + 'static>,
    ) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let source = MidiChunkSource::new(smf, note_source_spawner)?;

        Ok(Self {
            source: Box::new(source),
            has_finished: false,
        })
    }
}

impl<'a> AudioSource for MidiSource<'a> {
    fn on_note_on(&mut self, _key: u8) {
        self.has_finished = false;
    }

    fn on_note_off(&mut self, _key: u8) {
        self.has_finished = true;
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.source.fill_buffer(buffer);
    }
}
