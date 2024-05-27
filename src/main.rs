#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use qname::app::AppConfig;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn StdError>> {
    let args: Vec<String> = env::args().collect();
    let working_dir = fs::canonicalize(PathBuf::from(&args[1]))?;
    let mut schema_path = working_dir.clone();
    schema_path.push("schema.q");
    let schema = qname::fs::read_schema_file(&schema_path)?;
    AppConfig::run_with(schema, working_dir)
}
