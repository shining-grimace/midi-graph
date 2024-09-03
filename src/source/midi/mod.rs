pub mod chunk;
pub mod track;
pub mod util;

use crate::{
    util::{smf_from_file, soundfont_from_file, wav_from_file},
    BufferConsumer, Config, Error, FontSource, LfsrNoiseSource, MidiChunkSource, MidiDataSource,
    NoteEvent, NoteKind, NoteRange, SoundFont, SoundFontBuilder, SoundSource, SquareWaveSource,
    TriangleWaveSource,
};
use midly::Smf;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::source::log;

pub struct MidiSourceBuilder<'a> {
    smf: Smf<'a>,
    channel_fonts: HashMap<usize, SoundFont>,
}

impl<'a> MidiSourceBuilder<'a> {
    pub fn new(smf: Smf<'a>) -> Self {
        Self {
            smf,
            channel_fonts: HashMap::new(),
        }
    }

    pub fn add_channel_font(mut self, channel: usize, font: SoundFont) -> Self {
        self.channel_fonts.insert(channel, font);
        self
    }

    pub fn build(self) -> Result<MidiSource<'a>, Error> {
        MidiSource::new(self.smf, self.channel_fonts)
    }
}

pub struct MidiSource<'a> {
    source: Box<MidiChunkSource<'a>>,
    has_finished: bool,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, channel_fonts: HashMap<usize, SoundFont>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf);

        let source = MidiChunkSource::new(smf, channel_fonts)?;

        Ok(Self {
            source: Box::new(source),
            has_finished: false,
        })
    }

    pub fn from_config(config: Config) -> Result<Self, Error> {
        let smf = match config.midi {
            MidiDataSource::FilePath(file) => smf_from_file(file.as_str())?,
        };
        let mut channel_sources = HashMap::new();
        for (channel, font_source) in config.channels.iter() {
            match font_source {
                FontSource::Ranges(ranges) => {
                    let mut font_builder = SoundFontBuilder::new();
                    for range in ranges {
                        let note_range = NoteRange::new_inclusive_range(range.lower, range.upper);
                        match &range.source {
                            SoundSource::SquareWave(amplitude, duty_cycle) => {
                                font_builder = font_builder.add_range(note_range, || {
                                    Box::new(SquareWaveSource::new(*amplitude, *duty_cycle))
                                });
                            }
                            SoundSource::TriangleWave(amplitude) => {
                                font_builder = font_builder.add_range(note_range, || {
                                    Box::new(TriangleWaveSource::new(*amplitude))
                                });
                            }
                            SoundSource::LfsrNoise(
                                amplitude,
                                inside_feedback,
                                note_for_16_shifts,
                            ) => {
                                font_builder = font_builder.add_range(note_range, || {
                                    Box::new(LfsrNoiseSource::new(
                                        *amplitude,
                                        *inside_feedback,
                                        *note_for_16_shifts,
                                    ))
                                });
                            }
                            SoundSource::SampleFilePath(file_path, note) => {
                                font_builder = font_builder.add_range(note_range, || {
                                    Box::new(wav_from_file(file_path.as_str(), *note).unwrap())
                                });
                            }
                        };
                    }
                    let font = font_builder.build();
                    channel_sources.insert(*channel, font);
                }
                FontSource::Sf2FilePath(file_path, instrument) => {
                    let soundfont = soundfont_from_file(file_path.as_str(), *instrument)?;
                    channel_sources.insert(*channel, soundfont);
                }
            }
        }
        MidiSource::new(smf, channel_sources)
    }
}

impl<'a> BufferConsumer for MidiSource<'a> {
    fn set_note(&mut self, event: NoteEvent) {
        self.has_finished = match event.kind {
            NoteKind::NoteOn(_) => true,
            NoteKind::NoteOff(_) => false,
        };
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.source.fill_buffer(buffer);
    }
}
