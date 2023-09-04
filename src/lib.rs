
// Test suite for the Web and headless browsers.
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_tests;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
mod wasm_demo;

use midly::Smf;

pub enum Error {
    Io(std::io::Error),
    Midly(midly::Error)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<midly::Error> for Error {
    fn from(value: midly::Error) -> Self {
        Error::Midly(value)
    }
}

pub struct MidiProcessor {
    smf: Smf<'static>
}

impl MidiProcessor {

    pub fn from_file(file_name: &str) -> Result<MidiProcessor, Error> {
        let bytes = std::fs::read(file_name)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<MidiProcessor, Error> {
        let smf = Smf::parse(&bytes)?.to_static();
        Ok(MidiProcessor {
            smf
        })
    }
}
