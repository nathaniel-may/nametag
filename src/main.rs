#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use qname::gui;
use qname::schema::parse::parse;
use qname::schema::typecheck::typecheck;
use std::error::Error as StdError;

fn main() -> Result<(), Box<dyn StdError>> {
    let src = r#"schema "-" "_"
  [category "Media" (exactly 1) ['art', 'photo'/'ph', 'video'/'v']
  , category "People" (at_least 0) ['chris', 'nate', 'stefan']
  ]"#;
    let app = typecheck(parse(src)?)?.into();

    gui::run(app)?;
    Ok(())
}
