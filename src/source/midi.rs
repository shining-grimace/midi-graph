use crate::AudioSource;
use midly::Smf;

#[cfg(debug_assertions)]
use crate::source::log;

pub struct MidiSource<'a> {
    smf: Smf<'a>,
    source: Box<dyn AudioSource + Send + 'static>,
    has_finished: bool,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, source: Box<dyn AudioSource + Send + 'static>) -> Self {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        Self {
            smf,
            source,
            has_finished: false,
        }
    }
}

impl<'a> AudioSource for MidiSource<'a> {
    fn is_completed(&self) -> bool {
        self.has_finished
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        buffer.fill(0.0);
    }
}
