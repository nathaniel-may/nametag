use crate::{
    error::{Error, Result},
    schema::{self, Schema},
};
use std::{fs, path::Path};

pub fn read_schema_file(path: &Path) -> Result<Schema> {
    let contents = fs::read_to_string(path).map_err(Error::FailedToReadContents)?;
    let parsed = schema::parse::parse(&contents)?;
    let schema = schema::typecheck::typecheck(parsed)?;
    Ok(schema)
}
