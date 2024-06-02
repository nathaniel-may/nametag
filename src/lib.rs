pub mod app;
pub mod error;
pub mod filename;
pub mod fs;
pub mod schema;

use std::{env, path::PathBuf};

use app::AppConfig;
use error::{Error, Result};
use schema::{Category, Keyword};

type State = Vec<(Category, Vec<(Keyword, bool)>)>;

pub fn run() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_line_number(false)
        .with_thread_ids(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(Error::LoggerFailed)?;

    let args: Vec<String> = env::args().collect();
    // TODO use clap
    let input = &args.get(1).ok_or(Error::WorkingDirNotSpecified)?;
    let working_dir = std::fs::canonicalize(PathBuf::from(input))?;
    let mut schema_path = working_dir.clone();
    schema_path.push("schema.q");
    let schema = fs::read_schema_file(&schema_path)?;
    AppConfig::run_with(schema, working_dir)
}
