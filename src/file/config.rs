use crate::{Config, Error};
use ron::de::{from_bytes, from_reader};
use std::fs::File;

pub fn config_from_file(file_name: &str) -> Result<Config, Error> {
    let file = File::open(file_name)?;
    let config = from_reader(&file)?;
    Ok(config)
}

pub fn config_from_bytes(bytes: &[u8]) -> Result<Config, Error> {
    let config = from_bytes(bytes)?;
    Ok(config)
}
