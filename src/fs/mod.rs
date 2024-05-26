use crate::schema::{self, Schema};
use std::{error::Error, fs, path::Path};

pub fn read_schema_file(path: &Path) -> Result<Schema, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let parsed = schema::parse::parse(&contents)?;
    let schema = schema::typecheck::typecheck(parsed)?;
    Ok(schema)
}
