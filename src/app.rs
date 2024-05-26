#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{filename, schema::Schema, State};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Key, Label,
};
use rand::{rngs::ThreadRng, thread_rng};
use std::{
    borrow::Cow,
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
    pub file_id: String,
    pub ui_state: State,
    pub files: Vec<PathBuf>,
    pub rng: ThreadRng,
}

impl AppConfig {
    pub fn new(schema: Schema, working_dir: PathBuf) -> Self {
        let mut files = vec![];
        for path in read_dir(working_dir.clone()).unwrap() {
            // TODO filter out files that start with . and that are the schema file
            files.push(path.unwrap().path());
        }

        let ui_state = to_empty_state(&schema);
        let rng = thread_rng();

        let mut config = AppConfig {
            schema,
            ui_state,
            working_dir,
            active: 0,
            file_id: "".to_string(),
            files,
            rng,
        };
        config.gen_id();
        config
    }

    fn next(&mut self) {
        if self.active >= self.files.len() - 1 {
            self.active = 0;
        } else {
            self.active += 1;
        }
        self.gen_id()
    }

    fn prev(&mut self) {
        if self.active == 0 {
            self.active = self.files.len() - 1;
        } else {
            self.active -= 1;
        }
        self.gen_id()
    }

    fn gen_id(&mut self) {
        self.file_id = filename::gen_rand_id(&mut self.rng);
    }

    fn mk_filename(&self) -> String {
        let id = self.file_id.clone();
        let filename = filename::generate(&self.schema, &self.ui_state);
        let delim = self.schema.delim.clone();
        let ext = match self.active_file().extension() {
            Some(ext) => format!(".{}", ext.to_string_lossy()),
            None => String::new(),
        };
        match filename {
            Ok(name) => format!("{id}{delim}{name}{ext}"),
            Err(e) => e.to_string(),
        }
    }

    fn load_active(&self) -> Vec<u8> {
        let mut buffer = vec![];
        File::open(self.active_file())
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        buffer
    }

    fn active_uri(&self) -> String {
        let mut uri = "bytes://".to_string();
        uri.push_str(&self.active_file().to_string_lossy());
        uri
    }

    fn active_file(&self) -> &PathBuf {
        &self.files[self.active]
    }

    fn apply_rename(&mut self) {
        let mut to = self.working_dir.clone();
        to.push(self.mk_filename());
        println!("{:?}", self.active_file());
        println!("{:?}", to);
        std::fs::rename(self.active_file(), to.clone()).unwrap();

        // now that file has a different filename so we must update the system state of the folder
        // so the next refresh doesn't fail
        self.files[self.active] = to;
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

        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            self.apply_rename()
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
            ui.add(Label::new(format!(
                "filename: {}",
                self.active_file().file_name().unwrap().to_string_lossy()
            )));
            ui.add(Label::new(format!("new name: {}", self.mk_filename())));
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
