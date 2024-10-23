use crate::{Error, MidiSourceBuilder};
use midly::Smf;

pub fn midi_builder_from_file(file_name: &str) -> Result<MidiSourceBuilder, Error> {
    let bytes = std::fs::read(file_name)?;
    let midi_builder = midi_builder_from_bytes(&bytes)?;
    Ok(midi_builder)
}

pub fn midi_builder_from_bytes(bytes: &[u8]) -> Result<MidiSourceBuilder, Error> {
    let smf = Smf::parse(&bytes)?;
    let midi_builder = MidiSourceBuilder::new(smf);
    Ok(midi_builder)
}
