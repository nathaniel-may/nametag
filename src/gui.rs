#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::schema::{Category, Keyword, Schema};
use eframe::egui::{self, panel::Side};

pub fn run(app: App) -> Result<(), eframe::Error> {
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
pub struct App {
    pub state: Vec<(Category, Vec<(Keyword, bool)>)>,
}

impl From<Schema> for App {
    fn from(schema: Schema) -> Self {
        App {
            state: schema
                .categories
                .into_iter()
                .map(|(cat, kws)| (cat, kws.into_iter().map(|k| (k, false)).collect()))
                .collect(),
        }
    }
}

impl eframe::App for App {
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
