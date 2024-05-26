#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{filename, schema::Schema, State};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Key, Label,
};
use std::{
    fs::{read_dir, File},
    io::Read,
    path::PathBuf,
};

pub fn run(app: AppConfig) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "QName",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(app)
        }),
    )
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub working_dir: PathBuf,
    pub schema: Schema,
    pub active: usize,
    pub ui_state: State,
    pub files: Vec<PathBuf>,
}

impl AppConfig {
    pub fn new(schema: Schema, working_dir: PathBuf) -> Self {
        let mut files = vec![];
        for path in read_dir(working_dir.clone()).unwrap() {
            files.push(path.unwrap().path());
        }

        let ui_state = to_empty_state(&schema);

        AppConfig {
            schema,
            ui_state,
            working_dir,
            active: 0,
            files,
        }
    }

    fn next(&mut self) {
        if self.active >= self.files.len() - 1 {
            self.active = 0;
        } else {
            self.active += 1;
        }
    }

    fn prev(&mut self) {
        if self.active == 0 {
            self.active = self.files.len() - 1;
        } else {
            self.active -= 1;
        }
    }

    fn load_active(&self) -> Vec<u8> {
        let mut buffer = vec![];
        File::open(self.files[self.active].clone())
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        buffer
    }

    fn active_uri(&self) -> String {
        let mut uri = "bytes://".to_string();
        uri.push_str(&self.files[self.active].to_string_lossy());
        uri
    }
}

pub fn to_empty_state(schema: &Schema) -> State {
    schema
        .categories
        .clone()
        .into_iter()
        .map(|(cat, kws)| (cat, kws.into_iter().map(|k| (k, false)).collect()))
        .collect()
}

impl eframe::App for AppConfig {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            self.next();
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.prev();
        }

        egui::SidePanel::new(Side::Left, "keyword").show(ctx, |ui| {
            self.ui_state.iter_mut().for_each(|cat| {
                ui.label(cat.0.name.clone());
                cat.1.iter_mut().for_each(|kw| {
                    let name = kw.0.name.clone();
                    ui.checkbox(&mut kw.1, name);
                })
            })
        });

        egui::TopBottomPanel::new(TopBottomSide::Top, "filename").show(ctx, |ui| {
            let filename = filename::generate(&self.schema, &self.ui_state);
            let msg = match filename {
                Ok(name) => name,
                Err(e) => e.to_string(),
            };
            ui.add(Label::new(msg));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let uri = self.active_uri();
                // TODO I'm loading from disk on demand every time. figure out how to load them into the context
                // ctx.include_bytes(uri.clone(), self.load_active());
                let image = egui::Image::from_bytes(uri, self.load_active());
                ui.add(image.rounding(10.0));
            });
        });
    }
}
