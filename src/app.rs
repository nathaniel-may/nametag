#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{filename, schema::Schema, State};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Key, Label,
};
use rand::{rngs::ThreadRng, thread_rng};
use std::{
    fs::{read_dir, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
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
    pub runtime: Arc<tokio::runtime::Runtime>,
}

impl AppConfig {
    pub fn new(schema: Schema, working_dir: PathBuf) -> Self {
        let mut files = vec![];
        for path in read_dir(working_dir.clone()).unwrap() {
            let p = path.unwrap();
            let filename = p.file_name().to_string_lossy().to_string();
            if !filename.starts_with('.') && filename != "schema.q" {
                files.push(p.path());
            }
        }

        let ui_state = to_empty_state(&schema);
        let rng = thread_rng();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let runtime = Arc::new(runtime);

        let mut config = AppConfig {
            schema,
            ui_state,
            working_dir,
            active: 0,
            file_id: "".to_string(),
            files,
            rng,
            runtime,
        };
        config.gen_id();
        config
    }

    fn next(&mut self, ctx: &egui::Context) {
        if self.active >= self.files.len() - 1 {
            self.active = 0;
        } else {
            self.active += 1;
        }
        self.gen_id();

        // make sure the next few images are preloaded on separate threads
        let ahead = 3;
        let files = if self.active + ahead >= self.files.len() {
            [
                &self.files[self.active..],
                &self.files[..self.files.len() - self.active],
            ]
            .concat()
        } else {
            self.files[self.active..self.active + ahead].to_vec()
        };

        self.load_ahead(&files, ctx);
    }

    fn prev(&mut self, ctx: &egui::Context) {
        if self.active == 0 {
            self.active = self.files.len() - 1;
        } else {
            self.active -= 1;
        }
        self.gen_id();

        // make sure the previous few images are preloaded on separate threads
        let ahead = 3;
        let i = self.active as isize - ahead;
        let files = if i < 0 {
            [
                &self.files[..self.active],
                &self.files[self.files.len() - (-i as usize)..],
            ]
            .concat()
        } else {
            self.files[(i as usize)..self.active].to_vec()
        };

        self.load_ahead(&files, ctx);
    }

    // same as load, but spawns a new thread for each
    fn load_ahead(&self, paths: &[PathBuf], ctx: &egui::Context) {
        for path in paths {
            let ctx = ctx.clone();
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

    fn load_active(&self, ctx: &egui::Context) -> egui::Image {
        Self::load(self.active_file(), ctx);
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

    fn apply_rename(&mut self, ctx: &egui::Context) {
        let mut to = self.working_dir.clone();
        to.push(self.mk_filename());
        std::fs::rename(self.active_file(), to.clone()).unwrap();

        // the image will never be refrenced by its old name again so evict it from the cache
        ctx.forget_image(&Self::to_uri(self.active_file()));

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
            self.next(ctx);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.prev(ctx);
        }

        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            self.apply_rename(ctx)
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
                let image = self.load_active(ctx);
                ui.add(image.rounding(10.0));
            });
        });
    }
}
