use crate::{
    util, AsyncEventReceiver, BufferConsumerNode, CombinerSource, Config, Envelope, Error,
    EventChannel, Fader, FontSource, GraphLoader, LfsrNoiseSource, LoopRange, MidiDataSource,
    MixerSource, NoteRange, SawtoothWaveSource, SoundFontBuilder, SoundSource, SquareWaveSource,
    TriangleWaveSource,
};
use ron::de::{from_bytes, from_reader};
use std::fs::File;

#[derive(Default)]
pub struct FileGraphLoader;

impl FileGraphLoader {
    pub fn config_from_file(&self, file_name: &str) -> Result<Config, Error> {
        let file = File::open(file_name)?;
        let config = from_reader(&file)?;
        Ok(config)
    }

    pub fn config_from_bytes(&self, bytes: &[u8]) -> Result<Config, Error> {
        let config = from_bytes(bytes)?;
        Ok(config)
    }
}

impl GraphLoader for FileGraphLoader {
    fn prepare_source_recursive(&mut self, _source: &SoundSource) -> Result<(), Error> {
        Ok(())
    }

    fn load_source_recursive(
        &self,
        source: &SoundSource,
    ) -> Result<
        (
            Vec<EventChannel>,
            Box<dyn BufferConsumerNode + Send + 'static>,
        ),
        Error,
    > {
        let (event_channels, consumer) = match source {
            SoundSource::Midi {
                node_id,
                source,
                channels,
            } => {
                let mut midi_builder = match source {
                    MidiDataSource::FilePath(file) => {
                        util::midi_builder_from_file(*node_id, file.as_str())?
                    }
                };
                let mut event_channels = vec![];
                for (channel, source) in channels.iter() {
                    let (channels, font) = self.load_source_recursive(source)?;
                    event_channels.extend(channels);
                    midi_builder = midi_builder.add_channel_source(*channel, font);
                }
                let source = midi_builder.build()?;
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (event_channels, source)
            }
            SoundSource::EventReceiver { node_id, source } => {
                let (mut channels, source) = self.load_source_recursive(source)?;
                let (channel, source) = AsyncEventReceiver::new(*node_id, source);
                channels.push(channel);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (channels, source)
            }
            SoundSource::Font { node_id, config } => match config {
                FontSource::Ranges(ranges) => {
                    let mut all_channels = vec![];
                    let mut font_builder = SoundFontBuilder::new(*node_id);
                    for range in ranges {
                        let note_range = NoteRange::new_inclusive_range(range.lower, range.upper);
                        let (channels, source) = self.load_source_recursive(&range.source)?;
                        all_channels.extend(channels);
                        font_builder = font_builder.add_range(note_range, source)?;
                    }
                    let source: Box<dyn BufferConsumerNode + Send + 'static> =
                        Box::new(font_builder.build());
                    (all_channels, source)
                }
                FontSource::Sf2FilePath {
                    path,
                    instrument_index,
                } => {
                    let source =
                        util::soundfont_from_file(*node_id, path.as_str(), *instrument_index)?;
                    let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                    (vec![], source)
                }
            },
            SoundSource::SquareWave {
                node_id,
                amplitude,
                duty_cycle,
            } => {
                let source = SquareWaveSource::new(*node_id, *amplitude, *duty_cycle);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (vec![], source)
            }
            SoundSource::TriangleWave { node_id, amplitude } => {
                let source = TriangleWaveSource::new(*node_id, *amplitude);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (vec![], source)
            }
            SoundSource::SawtoothWave { node_id, amplitude } => {
                let source = SawtoothWaveSource::new(*node_id, *amplitude);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (vec![], source)
            }
            SoundSource::LfsrNoise {
                node_id,
                amplitude,
                inside_feedback,
                note_for_16_shifts,
            } => {
                let source = LfsrNoiseSource::new(
                    *node_id,
                    *amplitude,
                    *inside_feedback,
                    *note_for_16_shifts,
                );
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (vec![], source)
            }
            SoundSource::SampleFilePath {
                node_id,
                path,
                base_note,
                looping,
            } => {
                let loop_range = looping.as_ref().map(LoopRange::from_config);
                let source = util::wav_from_file(path.as_str(), *base_note, loop_range, *node_id)?;
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (vec![], source)
            }
            SoundSource::OneShotFilePath { node_id, path } => {
                let source = util::one_shot_from_file(path.as_str(), *node_id)?;
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (vec![], source)
            }
            SoundSource::Envelope {
                node_id,
                attack_time,
                decay_time,
                sustain_multiplier,
                release_time,
                source,
            } => {
                let (channels, source) = self.load_source_recursive(source)?;
                let source = Envelope::from_adsr(
                    *node_id,
                    *attack_time,
                    *decay_time,
                    *sustain_multiplier,
                    *release_time,
                    source,
                );
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (channels, source)
            }
            SoundSource::Combiner { node_id, sources } => {
                let mut event_channels: Vec<EventChannel> = vec![];
                let mut inner_sources: Vec<Box<dyn BufferConsumerNode + Send + 'static>> = vec![];
                for source in sources.iter() {
                    let (channels, source) = self.load_source_recursive(source)?;
                    event_channels.extend(channels);
                    inner_sources.push(source);
                }
                let source = CombinerSource::new(*node_id, inner_sources);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (event_channels, source)
            }
            SoundSource::Mixer {
                node_id,
                balance,
                source_0,
                source_1,
            } => {
                let (mut channels, source_0) = self.load_source_recursive(source_0)?;
                let (more_channels, source_1) = self.load_source_recursive(source_1)?;
                let source = MixerSource::new(*node_id, *balance, source_0, source_1);
                channels.extend(more_channels);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (channels, source)
            }
            SoundSource::Fader {
                node_id,
                initial_volume,
                source,
            } => {
                let (channels, source) = self.load_source_recursive(source)?;
                let source = Fader::new(*node_id, *initial_volume, source);
                let source: Box<dyn BufferConsumerNode + Send + 'static> = Box::new(source);
                (channels, source)
            }
        };
        Ok((event_channels, consumer))
    }
}
