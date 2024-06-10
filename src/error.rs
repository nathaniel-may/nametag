use std::{error::Error as StdError, fmt, io, result::Result as StdResult};
use tracing::subscriber::SetGlobalDefaultError;
use Error::*;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigParse(Box<serde_dhall::Error>),
    Eframe(eframe::Error),
    CantOpenWorkingDir(io::Error),
    WorkingDirScan(io::Error),
    EmptyWorkingDir,
    FailedRename(io::Error),
    FailedToOpen(io::Error),
    FailedToReadContents(io::Error),
    LoggerFailed(SetGlobalDefaultError),
    PathErr(io::Error),
    // TODO separate error type for config checking?
    EmptyStringNotValidTag,
    EmptyDelimiter,
    InvalidCharacterInTag(char),
    InvalidCharacterInDelim(char),
    DelimiterFoundInTag {
        category_name: String,
        tag: String,
    },
    CategoryWithNoTags {
        category_name: String,
    },
    TagsMustBeUnique {
        category_name: String,
        duplicated_tag: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigParse(e) => write!(f, "{e}"),
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
            FailedToReadContents(e) => write!(f, "Failed read file contents: {e}"),
            PathErr(e) => write!(f, "Issue with path: {e}"),
            EmptyStringNotValidTag => write!(
                f,
                "Schema cannot be configured with a tag that has no characters."
            ),
            CategoryWithNoTags { category_name } => {
                write!(f, "Category {category_name} has no tags.")
            }
            TagsMustBeUnique { category_name, duplicated_tag } => write!(f, "The tag \"{duplicated_tag}\" in category {category_name} has already been used in a prior category."),
            InvalidCharacterInTag(c) => write!(f, "Tags cannot contain the character {c}"),
            EmptyDelimiter => write!(f, "Delimiter cannot be the empty string."),
            InvalidCharacterInDelim(c) => write!(f, "Delimiters cannot contain the character {c}"),
            DelimiterFoundInTag { category_name, tag } => write!(f, "Tags cannot contain the specified delimiter. Change tag \"{tag}\" in category {category_name} to avoid the chosen delimiter."),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            EmptyWorkingDir
            | EmptyStringNotValidTag
            | CategoryWithNoTags { .. }
            | TagsMustBeUnique { .. }
            | InvalidCharacterInTag(_)
            | EmptyDelimiter
            | InvalidCharacterInDelim(_)
            | DelimiterFoundInTag { .. } => None,

            ConfigParse(e) => Some(e),
            Eframe(e) => Some(e),
            CantOpenWorkingDir(e) => Some(e),
            WorkingDirScan(e) => Some(e),
            FailedRename(e) => Some(e),
            FailedToOpen(e) => Some(e),
            LoggerFailed(e) => Some(e),
            FailedToReadContents(e) => Some(e),
            PathErr(e) => Some(e),
        }
    }
}

impl From<eframe::Error> for Error {
    fn from(e: eframe::Error) -> Self {
        Eframe(e)
    }
}
