mod util;

use crate::{AudioSource, Error};
use midly::Smf;

#[cfg(debug_assertions)]
use crate::source::log;

pub struct MidiSource<'a> {
    smf: Smf<'a>,
    source: Box<dyn AudioSource + Send + 'static>,
    has_finished: bool,
    event_index: usize,
    samples_per_tick: f64,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, source: Box<dyn AudioSource + Send + 'static>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let samples_per_tick = util::get_samples_per_tick(&smf)?;

        Ok(Self {
            smf,
            source,
            has_finished: false,
            event_index: 0,
            samples_per_tick,
        })
    }
}

impl<'a> AudioSource for MidiSource<'a> {
    fn is_completed(&self) -> bool {
        self.has_finished
    }

    fn rewind(&mut self) {
        self.has_finished = false;
        self.event_index = 0;
        self.source.rewind();
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.source.fill_buffer(buffer);
    }
}
