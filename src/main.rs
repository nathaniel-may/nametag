#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use qname::app::AppConfig;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    match qname::run() {
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
        Ok(()) => ExitCode::SUCCESS,
    }
}
