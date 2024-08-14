use crate::{util, BufferConsumer, Error, MidiTrackSource, NoteConsumer, NoteEvent};
use midly::Smf;
use std::sync::Arc;

pub struct MidiChunkSource<'a> {
    tracks: Vec<Box<MidiTrackSource<'a>>>,
}

impl<'a> MidiChunkSource<'a> {
    pub fn new(
        smf: Smf<'a>,
        note_consumer_spawner: fn() -> Box<dyn NoteConsumer + Send + 'static>,
    ) -> Result<Self, Error> {
        let samples_per_tick = util::get_samples_per_tick(&smf)?;
        let mut tracks = Vec::new();
        let smf_arc = Arc::new(smf);
        let track_count = smf_arc.tracks.len();
        for track_no in 0..track_count {
            let source = MidiTrackSource::new(
                Arc::clone(&smf_arc),
                track_no,
                samples_per_tick,
                note_consumer_spawner,
            );
            tracks.push(Box::new(source));
        }
        Ok(Self { tracks })
    }
}

impl<'a> BufferConsumer for MidiChunkSource<'a> {
    fn set_note(&mut self, _: NoteEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for track in self.tracks.iter_mut() {
            track.fill_buffer(buffer);
        }
    }
}
