mod range;

use crate::{
    util::{soundfont_from_file, wav_from_file},
    BufferConsumer, Envelope, Error, FontSource, LfsrNoiseSource, LoopRange, NoteConsumer,
    NoteEvent, NoteKind, NoteRange, SawtoothWaveSource, SoundSource, SquareWaveSource, Status,
    TriangleWaveSource,
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
        consumer: Box<dyn BufferConsumer + Send + 'static>,
    ) -> Result<Self, Error> {
        let mut consumers = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            consumers.push((0, consumer.duplicate()?));
        }
        self.ranges.push(RangeData::new(range, consumers));
        Ok(self)
    }

    pub fn build(self) -> SoundFont {
        SoundFont::new(self.ranges)
    }
}

pub struct SoundFont {
    ranges: Vec<RangeData>,
}

impl SoundFont {
    fn new(ranges: Vec<RangeData>) -> Self {
        Self { ranges }
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
    ) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        let consumer: Box<dyn BufferConsumer + Send + 'static> = match config {
            SoundSource::SquareWave {
                amplitude,
                duty_cycle,
            } => Box::new(SquareWaveSource::new(*amplitude, *duty_cycle)),
            SoundSource::TriangleWave { amplitude } => {
                Box::new(TriangleWaveSource::new(*amplitude))
            }
            SoundSource::SawtoothWave { amplitude } => {
                Box::new(SawtoothWaveSource::new(*amplitude))
            }
            SoundSource::LfsrNoise {
                amplitude,
                inside_feedback,
                note_for_16_shifts,
            } => Box::new(LfsrNoiseSource::new(
                *amplitude,
                *inside_feedback,
                *note_for_16_shifts,
            )),
            SoundSource::SampleFilePath {
                path,
                base_note,
                looping,
            } => {
                let loop_range = match looping {
                    Some(range) => Some(LoopRange::from_config(range)),
                    None => None,
                };
                Box::new(wav_from_file(path.as_str(), *base_note, loop_range)?)
            }
            SoundSource::Envelope {
                attack_time,
                decay_time,
                sustain_multiplier,
                release_time,
                source,
            } => {
                let consumer = Self::consumer_from_config(source)?;
                Box::new(Envelope::from_adsr(
                    *attack_time,
                    *decay_time,
                    *sustain_multiplier,
                    *release_time,
                    consumer,
                ))
            }
        };
        Ok(consumer)
    }
}

impl NoteConsumer for SoundFont {
    fn restart_with_event(&mut self, event: &NoteEvent) {
        let (note, vel) = match event.kind {
            NoteKind::NoteOn { note, vel } => (note, vel),
            NoteKind::NoteOff { note, vel } => (note, vel),
        };
        let turning_on = match event.kind {
            NoteKind::NoteOn { .. } => true,
            NoteKind::NoteOff { .. } => false,
        };
        if turning_on {
            for range_data in self.ranges.iter_mut() {
                range_data.turn_note_on(note, vel);
            }
        } else {
            for range_data in self.ranges.iter_mut() {
                range_data.turn_note_off(note, vel);
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        for range_data in self.ranges.iter_mut() {
            range_data.fill_buffer(buffer);
        }
        Status::Ok
    }
}
