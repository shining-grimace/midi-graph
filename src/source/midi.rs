use crate::AudioStreamer;
use midly::Smf;

pub struct MidiPlayer<'a> {
    smf: Smf<'a>,
    source: Box<dyn AudioStreamer + Send + 'static>,
}

impl<'a> MidiPlayer<'a> {
    pub fn new(smf: Smf<'a>, source: Box<dyn AudioStreamer + Send + 'static>) -> Self {
        Self { smf, source }
    }
}

impl<'a> AudioStreamer for MidiPlayer<'a> {
    fn is_completed(&self) -> bool {
        false
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {}
}
