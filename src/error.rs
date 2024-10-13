#[derive(Debug)]
pub enum Error {
    User(String),
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
    Midly(midly::Error),
    Hound(hound::Error),
    Soundfont(soundfont::error::ParseError),
    Cpal(cpal::BuildStreamError),
    NoDevice,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::User(e) => e.fmt(fmt),
            Error::Io(e) => e.fmt(fmt),
            Error::Ron(e) => e.fmt(fmt),
            Error::Midly(e) => e.fmt(fmt),
            Error::Hound(e) => e.fmt(fmt),
            Error::Soundfont(e) => fmt.write_fmt(format_args!("{:?}", e)),
            Error::Cpal(e) => e.fmt(fmt),
            Error::NoDevice => "No audio device available".fmt(fmt),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<ron::error::SpannedError> for Error {
    fn from(value: ron::error::SpannedError) -> Self {
        Error::Ron(value)
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
