mod range;

use crate::{
    util::{one_shot_from_file, soundfont_from_file, wav_from_file},
    BufferConsumer, BufferConsumerNode, Envelope, Error, Fader, FontSource, LfsrNoiseSource,
    LoopRange, MixerSource, Node, NodeEvent, NoteConsumer, NoteConsumerNode, NoteRange,
    SawtoothWaveSource, SoundSource, SquareWaveSource, TriangleWaveSource,
};
use range::RangeData;

const SOURCE_CAPACITY: usize = 8;

pub struct SoundFontBuilder {
    ranges: Vec<RangeData>,
}

impl SoundFontBuilder {
    pub fn new() -> Self {
        Self { ranges: vec![] }
    }

    pub fn add_range(
        mut self,
        range: NoteRange,
        consumer: Box<dyn BufferConsumerNode + Send + 'static>,
    ) -> Result<Self, Error> {
        let mut consumers = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            consumers.push(consumer.duplicate()?);
        }
        self.ranges.push(RangeData::new(range, consumers));
        Ok(self)
    }

    pub fn build(self) -> SoundFont {
        SoundFont::new(self.ranges)
    }
}

pub struct SoundFont {
    node_id: u64,
    ranges: Vec<RangeData>,
}

impl SoundFont {
    fn new(ranges: Vec<RangeData>) -> Self {
        Self {
            node_id: <Self as Node>::new_node_id(),
            ranges,
        }
    }

    pub fn from_config(config: &FontSource) -> Result<Self, Error> {
        match config {
            FontSource::Ranges(ranges) => {
                let mut font_builder = SoundFontBuilder::new();
                for range in ranges {
                    let note_range = NoteRange::new_inclusive_range(range.lower, range.upper);
                    let consumer = Self::consumer_from_config(&range.source)?;
                    font_builder = font_builder.add_range(note_range, consumer)?;
                }
                Ok(font_builder.build())
            }
            FontSource::Sf2FilePath {
                path,
                instrument_index,
            } => {
                let soundfont = soundfont_from_file(path.as_str(), *instrument_index)?;
                Ok(soundfont)
            }
        }
    }

    fn consumer_from_config(
        config: &SoundSource,
    ) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let consumer: Box<dyn BufferConsumerNode + Send + 'static> = match config {
            SoundSource::SquareWave {
                node_id,
                amplitude,
                duty_cycle,
            } => Box::new(SquareWaveSource::new(*node_id, *amplitude, *duty_cycle)),
            SoundSource::TriangleWave { node_id, amplitude } => {
                Box::new(TriangleWaveSource::new(*node_id, *amplitude))
            }
            SoundSource::SawtoothWave { node_id, amplitude } => {
                Box::new(SawtoothWaveSource::new(*node_id, *amplitude))
            }
            SoundSource::LfsrNoise {
                node_id,
                amplitude,
                inside_feedback,
                note_for_16_shifts,
            } => Box::new(LfsrNoiseSource::new(
                *node_id,
                *amplitude,
                *inside_feedback,
                *note_for_16_shifts,
            )),
            SoundSource::SampleFilePath {
                node_id,
                path,
                base_note,
                looping,
            } => {
                let loop_range = match looping {
                    Some(range) => Some(LoopRange::from_config(range)),
                    None => None,
                };
                Box::new(wav_from_file(
                    path.as_str(),
                    *base_note,
                    loop_range,
                    *node_id,
                )?)
            }
            SoundSource::OneShotFilePath { node_id, path } => {
                Box::new(one_shot_from_file(path.as_str(), *node_id)?)
            }
            SoundSource::Envelope {
                node_id,
                attack_time,
                decay_time,
                sustain_multiplier,
                release_time,
                source,
            } => {
                let consumer = Self::consumer_from_config(source)?;
                Box::new(Envelope::from_adsr(
                    *node_id,
                    *attack_time,
                    *decay_time,
                    *sustain_multiplier,
                    *release_time,
                    consumer,
                ))
            }
            SoundSource::Mixer {
                node_id,
                balance,
                source_0,
                source_1,
            } => {
                let consumer_0 = Self::consumer_from_config(source_0)?;
                let consumer_1 = Self::consumer_from_config(source_1)?;
                Box::new(MixerSource::new(*node_id, *balance, consumer_0, consumer_1))
            }
            SoundSource::Fader {
                node_id,
                initial_volume,
                source,
            } => {
                let consumer = Self::consumer_from_config(source)?;
                Box::new(Fader::new(*node_id, *initial_volume, consumer))
            }
        };
        Ok(consumer)
    }
}

impl NoteConsumerNode for SoundFont {}

impl Node for SoundFont {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, event: &NodeEvent) {
        for range_data in self.ranges.iter_mut() {
            range_data.on_event(event);
        }
    }
}

impl NoteConsumer for SoundFont {
    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for range_data in self.ranges.iter_mut() {
            range_data.fill_buffer(buffer);
        }
    }
}
