
use crate::Error;
use wav::{header::Header, bit_depth::BitDepth};
use std::{fs::File, io::Cursor, path::Path};

pub fn wav_from_file(file_name: &str) -> Result<(Header, BitDepth), Error> {
    let mut file = File::open(Path::new(file_name))?;
    let (header, data) = wav::read(&mut file)?;
    Ok((header, data))
}

pub fn wav_from_bytes(mut bytes: &[u8]) -> Result<(Header, BitDepth), Error> {
    let mut cursor = Cursor::new(bytes);
    let (header, data) = wav::read(&mut cursor)?;
    Ok((header, data))
}
