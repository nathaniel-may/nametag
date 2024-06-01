use crate::{error::Result, filename, schema::Schema, State};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Button, Color32, FontFamily, Hyperlink, Key, Label,
};
use rand::{rngs::ThreadRng, thread_rng};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::Arc,
};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub ctx: Arc<egui::Context>,
    pub working_dir: PathBuf,
    pub schema: Schema,
    pub active: usize,
    pub file_id: String,
    pub zoom: f32,
    pub ui_state: State,
    pub files: Vec<PathBuf>,
    pub rng: ThreadRng,
}

impl AppConfig {
    pub fn run_with(schema: Schema, working_dir: PathBuf) -> Result<()> {
        // TODO put this in crate::fs as its own function?
        // collect all the names of the files in the working dir so they can be loaded in the background
        let mut files = vec![];
        let dir = fs::read_dir(&working_dir).unwrap();
        for path in dir {
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

        let mut app = AppConfig {
            // dummy ctx that gets immediately overwritten.
            ctx: Arc::new(egui::Context::default()),
            schema,
            ui_state,
            working_dir,
            active: 0,
            file_id: "".to_string(),
            zoom: 1.0,
            files,
            rng,
        };
        app.gen_id();

        // create the ui

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
            ..Default::default()
        };

        // run the UI. Any errors returned from this function are fatal since the UI won't be created.
        eframe::run_native(
            "QName",
            options,
            Box::new(|cc| {
                // add the egui context to the app.
                // allows us to work with the cache without explicitly passing it around.
                app.ctx = Arc::new(cc.egui_ctx.clone());

                // set scale
                app.ctx.set_pixels_per_point(1.25);

                // set default styles
                app.ctx.style_mut(|style| {
                    style.override_font_id = Some(egui::FontId {
                        size: 16.0,
                        family: FontFamily::Proportional,
                    });
                });

                // add image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Box::new(app)
            }),
        )?;
        Ok(())
    }

    fn clear_state(&mut self) {
        self.ui_state = to_empty_state(&self.schema)
    }

    fn next(&mut self) {
        self.active = self.inc_file_index_by(1, self.active);
        self.zoom = 1.0;
        self.gen_id();
    }

    fn prev(&mut self) {
        self.active = self.dec_file_index_by(1, self.active);
        self.zoom = 1.0;
        self.gen_id();
    }

    fn inc_file_index_by(&self, n: usize, current: usize) -> usize {
        (current + n) % self.files.len()
    }

    fn dec_file_index_by(&self, n: usize, current: usize) -> usize {
        (current as isize - n as isize).rem_euclid(self.files.len() as isize) as usize
    }

    fn gen_id(&mut self) {
        self.file_id = filename::gen_rand_id(&mut self.rng);
    }

    fn mk_filename(&self) -> StdResult<String, String> {
        match filename::generate(&self.schema, &self.ui_state) {
            Ok(name) => {
                let id = self.file_id.clone();
                let delim = self.schema.delim.clone();
                let ext = match self.active_file().extension() {
                    Some(ext) => format!(".{}", ext.to_string_lossy()),
                    None => String::new(),
                };
                Ok(format!("{id}{delim}{name}{ext}"))
            }
            Err(e) => Err(e.to_string()),
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
        let uri = Self::to_uri(self.active_file());
        // skip the io if this uri is already in the cache
        if self.ctx.try_load_bytes(&uri).is_err() {
            let mut buffer = vec![];
            File::open(self.active_file())
                .unwrap()
                .read_to_end(&mut buffer)
                .unwrap();
            self.ctx.include_bytes(uri.clone(), buffer);
        }
        egui::Image::from_uri(uri)
    }

    fn apply_rename(&mut self) {
        // only apply the rename if there isn't an error generating the new filename
        if let Ok(filename) = self.mk_filename() {
            let mut to = self.working_dir.clone();
            to.push(filename);
            std::fs::rename(self.active_file(), to.clone()).unwrap();

            // the image will never be refrenced by its old name again so evict it from the cache
            self.ctx.forget_image(&Self::to_uri(self.active_file()));

            // update the list of filenames so the next refresh doesn't fail
            self.files[self.active] = to;
        }
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
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add(Label::new("Categories"));
                    let clear_button = ui
                        .add(Button::new("Clear"))
                        .on_hover_text("Clear all checkboxes");

                    if clear_button.clicked() {
                        self.clear_state();
                    }
                });
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                self.ui_state.iter_mut().for_each(|cat| {
                    ui.label(cat.0.name.clone());
                    cat.1.iter_mut().for_each(|kw| {
                        let name = kw.0.name.clone();
                        ui.checkbox(&mut kw.1, name);
                    })
                })
            });
        });

        egui::TopBottomPanel::new(TopBottomSide::Top, "filename").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new("filename:"));

                let filename = self
                    .active_file()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                ui.add(Hyperlink::from_label_and_url(
                    &filename,
                    format!(
                        "file://{}/{}",
                        self.working_dir.to_str().unwrap(),
                        &filename
                    ),
                ));
            });

            match self.mk_filename() {
                Ok(name) => {
                    ui.add(Label::new(format!("new name: {name}",)));
                }
                Err(msg) => {
                    ui.horizontal(|ui| {
                        ui.visuals_mut().override_text_color = Some(Color32::RED);
                        ui.add(Label::new(format!("schema error: {msg}",)))
                    });
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.zoom *= ctx.input(|i| i.zoom_delta());

            egui::ScrollArea::both().show(ui, |ui| {
                let image = self
                    .load_active()
                    .rounding(10.0)
                    .fit_to_fraction(egui::Vec2 {
                        x: self.zoom,
                        y: self.zoom,
                    });

                ui.add(image);
            });
        });
    }
}
