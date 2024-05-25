#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use qname::schema::{Category, Keyword, Requirement};
use qname::{gui, gui::App};

fn main() -> Result<(), eframe::Error> {
    let media = Category {
        name: "Media".to_string(),
        id: "m".to_string(),
        requirement: Requirement::Exactly(1),
    };

    let app = App {
        state: vec![(
            media,
            vec![
                (
                    Keyword {
                        name: "Art".to_string(),
                        id: "r".to_string(),
                    },
                    false,
                ),
                (
                    Keyword {
                        name: "Photo".to_string(),
                        id: "ph".to_string(),
                    },
                    true,
                ),
            ],
        )],
    };

    gui::run(app)
}
