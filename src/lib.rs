pub mod app;
pub mod error;
pub mod filename;
pub mod fs;
pub mod schema;

use std::{env, path::PathBuf};

use app::AppConfig;
use error::Result;
use schema::{Category, Keyword};

type State = Vec<(Category, Vec<(Keyword, bool)>)>;

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let working_dir = std::fs::canonicalize(PathBuf::from(&args[1]))?;
    let mut schema_path = working_dir.clone();
    schema_path.push("schema.q");
    let schema = fs::read_schema_file(&schema_path)?;
    AppConfig::run_with(schema, working_dir)
}
