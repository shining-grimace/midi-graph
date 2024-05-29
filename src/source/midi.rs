use crate::AudioSource;
use midly::Smf;

pub struct MidiSource<'a> {
    smf: Smf<'a>,
    source: Box<dyn AudioSource + Send + 'static>,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, source: Box<dyn AudioSource + Send + 'static>) -> Self {
        Self { smf, source }
    }
}

impl<'a> AudioSource for MidiSource<'a> {
    fn is_completed(&self) -> bool {
        false
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {}
}
