use std::{error::Error as StdError, fmt, io, result::Result as StdResult};
use Error::*;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    FS(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FS(e) => write!(f, "file system error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        FS(e)
    }
}
