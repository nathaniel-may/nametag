use crate::{
    error::{Error, Result},
    schema::{self, Schema},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn read_schema_file(path: &Path) -> Result<Schema> {
    let contents = fs::read_to_string(path).map_err(Error::FailedToReadContents)?;
    let parsed = schema::parse::parse(&contents)?;
    let schema = schema::typecheck::typecheck(parsed)?;
    Ok(schema)
}

/// collects filenames of all non-directory entries in the given directory.
pub fn collect_filenames(dir: &dyn AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = vec![];
    let dir = fs::read_dir(dir).map_err(Error::CantOpenWorkingDir)?;
    for path in dir {
        let entry = path.map_err(Error::WorkingDirScan)?;
        // skip sub directories
        if !entry.path().is_dir() {
            files.push(entry.path());
        }
    }

    Ok(files)
}
