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
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub ctx: Arc<egui::Context>,
    pub working_dir: PathBuf,
    pub schema: Schema,
    pub active: usize,
    pub file_id: String,
    pub ui_state: State,
    pub files: Vec<PathBuf>,
    // None for infinite cache and preloading everything
    pub file_cache_size: Option<usize>,
    pub rng: ThreadRng,
    pub runtime: Arc<tokio::runtime::Handle>,
}

impl AppConfig {
    pub async fn run_with(schema: Schema, working_dir: PathBuf) -> Result<(), Box<dyn StdError>> {
        // collect all the names of the files in the working dir so they can be loaded in the background
        let mut files = vec![];
        let mut entries = fs::read_dir(&working_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            // TODO skip directories
            let filename = entry.file_name().to_string_lossy().to_string();
            // skip dotfiles and our schema file
            if !filename.starts_with('.') && filename != "schema.q" {
                files.push(entry.path());
            }
        }

        let ui_state = to_empty_state(&schema);
        let rng = thread_rng();
        let runtime = tokio::runtime::Handle::current();
        let runtime = Arc::new(runtime);
        let file_cache_size = if 30 < files.len() { Some(30) } else { None };

        let mut app = AppConfig {
            // dummy ctx that gets immediately overwritten.
            ctx: Arc::new(egui::Context::default()),
            schema,
            ui_state,
            working_dir,
            active: 0,
            file_id: "".to_string(),
            files,
            file_cache_size,
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
                if let Some(cache_size) = app.file_cache_size {
                    // load upto the configured max
                    let prev = cache_size / 2;
                    let next = cache_size - prev;
                    app.load_ahead(&app.files[..next]);
                    // load from the current picture in the direction the user would scroll
                    let mut backwards: Vec<PathBuf> = app.files[app.files.len() - prev..].to_vec();
                    backwards.reverse();
                    app.load_ahead(&backwards);
                } else {
                    // load everything
                    app.load_ahead(&app.files);
                }

                // add image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Box::new(app)
            }),
        )?;
        Ok(())
    }

    fn next(&mut self) {
        self.active = self.inc_file_index_by(1, self.active);
        self.gen_id();

        // If there's a limited cache, preload the next one out, and drop the last one
        if let Some(cache_size) = self.file_cache_size {
            let prev = cache_size / 2;
            let next = cache_size - prev;
            let i = self.inc_file_index_by(next, self.active);
            self.load_ahead(&[self.files[i].clone()]);
            self.ctx.forget_image(&Self::to_uri(&self.files[prev]))
        }
    }

    fn prev(&mut self) {
        self.active = self.dec_file_index_by(1, self.active);
        self.gen_id();

        // If there's a limited cache, preload the next one out, and drop the last one
        if let Some(cache_size) = self.file_cache_size {
            let prev = cache_size / 2;
            let next = cache_size - prev;
            let i = self.inc_file_index_by(prev, self.active);
            self.load_ahead(&[self.files[i].clone()]);
            self.ctx.forget_image(&Self::to_uri(&self.files[next]))
        }
    }

    fn inc_file_index_by(&self, n: usize, current: usize) -> usize {
        (current + n) % self.files.len()
    }

    fn dec_file_index_by(&self, n: usize, current: usize) -> usize {
        (current as isize - n as isize).rem_euclid(self.files.len() as isize) as usize
    }

    // same as load, but spawns a new thread for the lot
    fn load_ahead(&self, paths: &[PathBuf]) {
        let ctx = self.ctx.clone();
        let paths: Vec<PathBuf> = paths.to_vec();
        self.runtime.spawn(async move {
            for path in paths {
                Self::load(&path, &ctx).await;
            }
        });
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                Self::load(self.active_file(), &self.ctx).await;
                egui::Image::from_uri(Self::to_uri(self.active_file()))
            })
        })
    }

    // loads bytes from file into the context
    async fn load(path: &Path, ctx: &egui::Context) {
        let uri = Self::to_uri(path);
        // skip if this uri is already in the cache
        if ctx.try_load_bytes(&uri).is_err() {
            let mut buffer = vec![];
            File::open(path)
                .await
                .unwrap()
                .read_to_end(&mut buffer)
                .await
                .unwrap();
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
