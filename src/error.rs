
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Midly(midly::Error),
    Cpal(cpal::BuildStreamError),
    NoDevice
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

impl From<cpal::BuildStreamError> for Error {
    fn from(value: cpal::BuildStreamError) -> Self {
        Error::Cpal(value)
    }
}
