use crate::schema::{SchemaParseError, SchemaTypeCheckError};
use std::{error::Error as StdError, fmt, io, result::Result as StdResult};
use tracing::subscriber::SetGlobalDefaultError;
use Error::*;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    // TODO remove this one for app=specific file system failures
    Fs(io::Error),
    Parse(SchemaParseError),
    Typecheck(SchemaTypeCheckError),
    Eframe(eframe::Error),
    CantOpenWorkingDir(io::Error),
    WorkingDirScan(io::Error),
    EmptyWorkingDir,
    FailedRename(io::Error),
    FailedToOpen(io::Error),
    LoggerFailed(SetGlobalDefaultError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fs(e) => write!(f, "file system error: {e}"),
            Parse(e) => write!(f, "{e}"),
            Typecheck(e) => write!(f, "{e}"),
            Eframe(e) => write!(f, "{e}"),
            CantOpenWorkingDir(e) => write!(f, "Cannot open working directory: {e}"),
            WorkingDirScan(e) => write!(
                f,
                "Encountered an error while scanning the working directory: {e}"
            ),
            EmptyWorkingDir => write!(f, "Working directory has nothing to work with"),
            FailedRename(e) => write!(f, "Failed rename: {e}"),
            FailedToOpen(e) => write!(f, "Failed to open file: {e}"),
            LoggerFailed(e) => write!(f, "Failed to set up logger: {e}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            EmptyWorkingDir => None,
            Fs(e) => Some(e),
            Parse(e) => Some(e),
            Typecheck(e) => Some(e),
            Eframe(e) => Some(e),
            CantOpenWorkingDir(e) => Some(e),
            WorkingDirScan(e) => Some(e),
            FailedRename(e) => Some(e),
            FailedToOpen(e) => Some(e),
            LoggerFailed(e) => Some(e),
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
