#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{filename, schema::Schema, State};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Key, Label,
};
use rand::{rngs::ThreadRng, thread_rng};
use std::{
    error::Error as StdError,
    fs::{read_dir, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub ctx: Arc<egui::Context>,
    pub working_dir: PathBuf,
    pub schema: Schema,
    pub active: usize,
    pub file_id: String,
    pub ui_state: State,
    pub files: Vec<PathBuf>,
    pub rng: ThreadRng,
    pub runtime: Arc<tokio::runtime::Runtime>,
}

impl AppConfig {
    pub fn run_with(schema: Schema, working_dir: PathBuf) -> Result<(), Box<dyn StdError>> {
        println!("Run With");
        // collect all the names of the files in the working dir so they can be loaded in the background
        let mut files = vec![];
        for path in read_dir(working_dir.clone()).unwrap() {
            // TODO skip directories
            let p = path.unwrap();
            let filename = p.file_name().to_string_lossy().to_string();
            // skip dotfiles and our schema file
            if !filename.starts_with('.') && filename != "schema.q" {
                files.push(p.path());
            }
        }

        let ui_state = to_empty_state(&schema);
        let rng = thread_rng();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let runtime = Arc::new(runtime);

        let mut app = AppConfig {
            // dummy ctx that gets immediately overwritten.
            ctx: Arc::new(egui::Context::default()),
            schema,
            ui_state,
            working_dir,
            active: 0,
            file_id: "".to_string(),
            files,
            rng,
            runtime,
        };
        app.gen_id();

        // create the ui

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 800.0]),
            ..Default::default()
        };

        // run the UI
        eframe::run_native(
            "QName",
            options,
            Box::new(|cc| {
                // add the egui context to the app.
                // allows us to work with the cache without explicitly passing it around.
                app.ctx = Arc::new(cc.egui_ctx.clone());

                // prepopulate cache on separate threads
                let prepop: Vec<PathBuf> = [app.files.first(), app.files.get(1), app.files.last()]
                    .into_iter()
                    .filter_map(|x| x.cloned())
                    .collect();
                app.load_ahead(&prepop);

                // add image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Box::new(app)
            }),
        )?;
        Ok(())
    }

    fn next(&mut self) {
        if self.active >= self.files.len() - 1 {
            self.active = 0;
        } else {
            self.active += 1;
        }
        self.gen_id();

        // we're preloading the next one every time we progress.
        let i = (self.active + 1) % self.files.len();
        self.load_ahead(&[self.files[i].clone()]);
    }

    fn prev(&mut self) {
        if self.active == 0 {
            self.active = self.files.len() - 1;
        } else {
            self.active -= 1;
        }
        self.gen_id();

        // we're preloading the previous one every time we progress.
        let i = (self.active as isize - 1).rem_euclid(self.files.len() as isize) as usize;
        self.load_ahead(&[self.files[i].clone()]);
    }

    // same as load, but spawns a new thread for each
    fn load_ahead(&self, paths: &[PathBuf]) {
        for path in paths {
            let ctx = self.ctx.clone();
            let path = path.clone();
            self.runtime.spawn(async move {
                Self::load(&path, &ctx);
            });
        }
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

    fn to_uri(path: &Path) -> String {
        let mut uri = "bytes://".to_string();
        uri.push_str(&path.to_string_lossy());
        uri
    }

    fn active_file(&self) -> &PathBuf {
        &self.files[self.active]
    }

    fn load_active(&self) -> egui::Image {
        Self::load(self.active_file(), &self.ctx);
        egui::Image::from_uri(Self::to_uri(self.active_file()))
    }

    // loads bytes from file into the context
    fn load(path: &Path, ctx: &egui::Context) {
        let uri = Self::to_uri(path);
        // skip if this uri is already in the cache
        if ctx.try_load_bytes(&uri).is_err() {
            let mut buffer = vec![];
            File::open(path).unwrap().read_to_end(&mut buffer).unwrap();
            ctx.include_bytes(uri.clone(), buffer);
        }
    }

    fn apply_rename(&mut self) {
        let mut to = self.working_dir.clone();
        to.push(self.mk_filename());
        std::fs::rename(self.active_file(), to.clone()).unwrap();

        // the image will never be refrenced by its old name again so evict it from the cache
        self.ctx.forget_image(&Self::to_uri(self.active_file()));

        // update the list of filenames so the next refresh doesn't fail
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
            self.prev();
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.next();
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
                let image = self.load_active();
                ui.add(image.rounding(10.0));
            });
        });
    }
}
