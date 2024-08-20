use crate::Error;
use midly::Smf;

pub fn smf_from_file(file_name: &str) -> Result<Smf<'static>, Error> {
    let bytes = std::fs::read(file_name)?;
    smf_from_bytes(&bytes)
}

pub fn smf_from_bytes(bytes: &[u8]) -> Result<Smf<'static>, Error> {
    let smf = Smf::parse(&bytes)?.to_static();
    Ok(smf)
}
