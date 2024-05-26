#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use qname::gui;
use std::error::Error as StdError;
use std::path::Path;

fn main() -> Result<(), Box<dyn StdError>> {
    let app = qname::fs::read_schema_file(Path::new("./ignore/schema.q"))?.into();
    gui::run(app)?;
    Ok(())
}
