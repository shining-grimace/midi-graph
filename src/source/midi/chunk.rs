use crate::{util, BufferConsumer, Error, MidiTrackSource, NoteEvent, SoundFont, Status};
use midly::Smf;
use std::{collections::HashMap, sync::Arc};

pub struct MidiChunkSource<'a> {
    channel_sources: HashMap<usize, Box<MidiTrackSource<'a>>>,
}

impl<'a> MidiChunkSource<'a> {
    pub fn new(smf: Smf<'a>, channel_fonts: HashMap<usize, SoundFont>) -> Result<Self, Error> {
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
        Ok(Self { channel_sources })
    }
}

impl<'a> BufferConsumer for MidiChunkSource<'a> {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        Err(Error::User(
            "MIDI chunk source cannot be replicated".to_owned(),
        ))
    }

    fn set_note(&mut self, _: NoteEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        let mut status = Status::Ok;
        for (_, source) in self.channel_sources.iter_mut() {
            match source.fill_buffer(buffer) {
                Status::Ok => {}
                Status::Ended => {
                    status = Status::Ended;
                }
            };
        }
        status
    }
}
