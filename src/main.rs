#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, panel::Side};
use qname::{Category, Keyword};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let media = Category {
        name: "Media".to_string(),
        id: "m".to_string(),
        requirement: Some(1),
    };

    let app = MyApp {
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(app)
        }),
    )
}

#[derive(Clone, Debug, Default)]
struct MyApp {
    state: Vec<(Category, Vec<(Keyword, bool)>)>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Left, "keyword").show(ctx, |ui| {
            self.state.iter_mut().for_each(|cat| {
                ui.label(cat.0.name.clone());
                cat.1.iter_mut().for_each(|kw| {
                    let name = kw.0.name.clone();
                    ui.checkbox(&mut kw.1, name);
                })
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::Image::new("https://picsum.photos/seed/1.759706314/1024").rounding(10.0),
                );
            });
        });
    }
}
