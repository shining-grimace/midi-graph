use crate::{util, AudioSource, Error, MidiTrackSource};
use midly::Smf;
use std::sync::Arc;

pub struct MidiChunkSource<'a> {
    smf: Arc<Smf<'a>>,
    tracks: Vec<Box<MidiTrackSource<'a>>>,
}

impl<'a> MidiChunkSource<'a> {
    pub fn new(
        smf: Smf<'a>,
        note_source_spawner: fn() -> Box<dyn AudioSource + Send + 'static>,
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
                note_source_spawner,
            );
            tracks.push(Box::new(source));
        }
        Ok(Self {
            smf: smf_arc,
            tracks,
        })
    }
}

impl<'a> AudioSource for MidiChunkSource<'a> {
    fn on_note_on(&mut self, key: u8) {}

    fn on_note_off(&mut self, key: u8) {}

    fn fill_buffer(&mut self, key: u8, buffer: &mut [f32]) {
        for track in self.tracks.iter_mut() {
            track.fill_buffer(key, buffer);
        }
    }
}
