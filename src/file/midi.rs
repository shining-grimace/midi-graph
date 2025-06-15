use crate::{Error, midi::MidiNodeBuilder};
use midly::Smf;

pub fn midi_builder_from_file(
    node_id: Option<u64>,
    file_name: &str,
) -> Result<MidiNodeBuilder, Error> {
    let bytes = std::fs::read(file_name)?;
    midi_builder_from_bytes(node_id, &bytes)
}

pub fn midi_builder_from_bytes(
    node_id: Option<u64>,
    bytes: &[u8],
) -> Result<MidiNodeBuilder, Error> {
    let smf = Smf::parse(bytes)?;
    let midi_builder = MidiNodeBuilder::new(node_id, smf)?;
    Ok(midi_builder)
}
