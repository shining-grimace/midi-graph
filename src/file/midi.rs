use crate::{Error, MidiSourceBuilder};
use midly::Smf;

pub fn midi_builder_from_file(
    node_id: Option<u64>,
    file_name: &str,
) -> Result<MidiSourceBuilder, Error> {
    let bytes = std::fs::read(file_name)?;
    midi_builder_from_bytes(node_id, &bytes)
}

pub fn midi_builder_from_bytes(
    node_id: Option<u64>,
    bytes: &[u8],
) -> Result<MidiSourceBuilder, Error> {
    let smf = Smf::parse(bytes)?;
    let midi_builder = MidiSourceBuilder::new(node_id, smf)?;
    Ok(midi_builder)
}
