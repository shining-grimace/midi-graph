
#[cfg(test)]
mod tests;

use midly::Smf;

enum Error {
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

struct MidiProcessor {
    smf: Smf<'static>
}

impl MidiProcessor {
    pub fn new(file_name: &str) -> Result<MidiProcessor, Error> {
        let bytes = std::fs::read(file_name)?;
        let smf = Smf::parse(&bytes)?.to_static();
        Ok(MidiProcessor {
            smf
        })
    }
}
