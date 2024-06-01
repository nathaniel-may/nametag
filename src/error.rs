use crate::schema::{SchemaParseError, SchemaTypeCheckError};
use std::{error::Error as StdError, fmt, io, result::Result as StdResult};
use Error::*;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Fs(io::Error),
    Parse(SchemaParseError),
    Typecheck(SchemaTypeCheckError),
    Eframe(eframe::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fs(e) => write!(f, "file system error: {}", e),
            Parse(e) => write!(f, "{}", e),
            Typecheck(e) => write!(f, "{}", e),
            Eframe(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Fs(e) => Some(e),
            Parse(e) => Some(e),
            Typecheck(e) => Some(e),
            Eframe(e) => Some(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Fs(e)
    }
}

impl From<SchemaParseError> for Error {
    fn from(e: SchemaParseError) -> Self {
        Parse(e)
    }
}

impl From<SchemaTypeCheckError> for Error {
    fn from(e: SchemaTypeCheckError) -> Self {
        Typecheck(e)
    }
}

impl From<eframe::Error> for Error {
    fn from(e: eframe::Error) -> Self {
        Eframe(e)
    }
}
