use crate::{config::SoundSource, BufferConsumerNode, Error, EventChannel, FontSource};

pub trait GraphLoader {
    fn prepare_source_recursive(&mut self, source: &SoundSource);

    fn load_source_recursive(
        &self,
        source: &SoundSource,
    ) -> Result<
        (
            Vec<EventChannel>,
            Box<dyn BufferConsumerNode + Send + 'static>,
        ),
        Error,
    >;

    fn traverse_sources(root: &SoundSource, mut yield_source: impl FnMut(&SoundSource)) {
        yield_source(root);
        match root {
            SoundSource::Midi { channels, .. } => {
                for channel in channels.iter() {
                    yield_source(channel.1);
                }
            }
            SoundSource::EventReceiver { source, .. } => {
                yield_source(source.as_ref());
            }
            SoundSource::Font { config, .. } => match config {
                FontSource::Ranges(ranges) => {
                    for range in ranges.iter() {
                        yield_source(&range.source);
                    }
                }
                FontSource::Sf2FilePath { .. } => {}
            },
            SoundSource::SquareWave { .. } => {}
            SoundSource::TriangleWave { .. } => {}
            SoundSource::SawtoothWave { .. } => {}
            SoundSource::LfsrNoise { .. } => {}
            SoundSource::SampleFilePath { .. } => {}
            SoundSource::OneShotFilePath { .. } => {}
            SoundSource::Envelope { source, .. } => {
                yield_source(source);
            }
            SoundSource::Combiner { sources, .. } => {
                for source in sources.iter() {
                    yield_source(source);
                }
            }
            SoundSource::Mixer {
                source_0, source_1, ..
            } => {
                yield_source(source_0);
                yield_source(source_1);
            }
            SoundSource::Fader { source, .. } => {
                yield_source(source);
            }
        }
    }
}
