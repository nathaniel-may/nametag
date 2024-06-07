pub mod app;
pub mod error;
pub mod filename;
pub mod fs;
pub mod schema;

use app::App;
use clap::Parser;
use error::{Error, Result};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
struct Args {
    working_dir: PathBuf,
}

pub fn run() -> Result<()> {
    // parse command line args
    let args = Args::parse();

    // set up logging
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_line_number(false)
        .with_thread_ids(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(Error::LoggerFailed)?;

    // run the app
    let working_dir = std::fs::canonicalize(args.working_dir).map_err(Error::PathErr)?;
    let mut schema_path = working_dir.clone();
    schema_path.push("schema.dhall");
    let schema = fs::read_schema_file(&schema_path)?;
    App::run_with(schema, working_dir)
}
