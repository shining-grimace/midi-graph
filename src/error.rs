#[derive(Debug)]
pub enum Error {
    User(String),
    Io(std::io::Error),
    Midly(midly::Error),
    Hound(hound::Error),
    Soundfont(soundfont::error::ParseError),
    Cpal(cpal::BuildStreamError),
    NoDevice,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<hound::Error> for Error {
    fn from(value: hound::Error) -> Self {
        Error::Hound(value)
    }
}

impl From<midly::Error> for Error {
    fn from(value: midly::Error) -> Self {
        Error::Midly(value)
    }
}

impl From<soundfont::error::ParseError> for Error {
    fn from(value: soundfont::error::ParseError) -> Self {
        Error::Soundfont(value)
    }
}

impl From<cpal::BuildStreamError> for Error {
    fn from(value: cpal::BuildStreamError) -> Self {
        Error::Cpal(value)
    }
}
