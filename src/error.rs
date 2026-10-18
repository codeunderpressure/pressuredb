use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
#[allow(dead_code)]
pub enum DbError {
    KeyNotFound,
    PageFull,
    KeyTooLarge,
    ValTooLarge,
    CorruptedPage,
    Io(io::Error),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyNotFound => write!(f, "key not found"),
            Self::PageFull => write!(f, "node size exceeds 4096 bytes"),
            Self::KeyTooLarge => write!(f, "key length exceeds maximum allowed"),
            Self::ValTooLarge => write!(f, "value length exceeds maximum allowed"),
            Self::CorruptedPage => write!(f, "corrupted page data"),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DbError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
