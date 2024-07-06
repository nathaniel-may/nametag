use crate::{
    error::{Error, Result},
    filename::{self, gen_salt},
    fs_util,
    schema::{self, Schema},
};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Button, Color32, FontFamily, Key, Label,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::Arc,
};
use tracing::{error, info};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiBlock {
    Salt {
        value: String,
        definition: schema::Salt,
    },
    Category {
        name: String,
        values: Vec<(String, bool)>,
    },
}

#[derive(Clone, Debug)]
pub struct App {
    pub ctx: Arc<egui::Context>,
    pub working_dir: PathBuf,
    pub schema: Schema,
    pub active: usize,
    pub zoom: f32,
    pub ui_state: Vec<UiBlock>,
    pub files: Vec<PathBuf>,
    pub rng: ChaCha8Rng,
    pub parsed_counts: HashMap<String, usize>,
}

impl App {
    pub fn run_with(schema: Schema, working_dir: PathBuf) -> Result<()> {
        info!("Reading working directory");
        let files: Vec<PathBuf> = fs_util::collect_filenames(&working_dir)?
            .into_iter()
            .filter(|path| {
                // since this string representation is only used to rule out certain files, it's safe to use even in cross-platform builds
                let filename = path
                    .file_name()
                    .map_or(String::new(), |fname| fname.to_string_lossy().to_string());
                // skip dotfiles and our schema file
                !filename.starts_with('.') && filename != "schema.dhall"
            })
            .collect();

        // UI must display the first image. Exit if there's nothing in the directory.
        if files.is_empty() {
            return Err(Error::EmptyWorkingDir);
        }

        let mut rng = ChaCha8Rng::from_entropy();
        let ui_state = to_empty_state(&schema, &mut rng);
        let parsed_counts = App::reparse_recount(&schema, &files);

        let mut app = App {
            // dummy ctx that gets immediately overwritten.
            ctx: Arc::new(egui::Context::default()),
            schema,
            ui_state,
            working_dir,
            active: 0,
            zoom: 1.0,
            files,
            rng,
            parsed_counts,
        };

        info!("Building the UI");
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
            ..Default::default()
        };

        // run the UI. Any errors returned from this function are fatal since the UI won't be created.
        eframe::run_native(
            "Nametag",
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

    fn reparse_recount(schema: &Schema, files: &[PathBuf]) -> HashMap<String, usize> {
        let mut m = HashMap::new();
        for file in files {
            let name = file.file_stem().unwrap().to_string_lossy();
            match schema.parse(&name) {
                Err(_) => {
                    *m.entry("__error".to_string()).or_insert(0) += 1;
                }
                Ok(blocks) => {
                    for block in blocks {
                        match block {
                            UiBlock::Salt { .. } => (),
                            UiBlock::Category { values, .. } => {
                                for (tag, present) in values {
                                    if present {
                                        *m.entry(tag.clone()).or_insert(0) += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        m
    }

    fn clear_state(&mut self) {
        self.ui_state = to_empty_state(&self.schema, &mut self.rng)
    }

    fn next(&mut self) {
        self.active = self.inc_file_index_by(1, self.active);
        self.zoom = 1.0;
        // attempt to parse the next file name which will include the salts
        if !self.parse_current_file() {
            // if that failed, generate new salts for the filename
            self.ui_state.iter_mut().for_each(|block| match block {
                UiBlock::Salt { value, definition } => *value = gen_salt(definition, &mut self.rng),
                UiBlock::Category { .. } => (),
            });
        }
    }

    fn prev(&mut self) {
        self.active = self.dec_file_index_by(1, self.active);
        self.zoom = 1.0;
        // attempt to parse the previous file name which will include the salts
        if !self.parse_current_file() {
            // if that failed, generate new salts for the filename
            self.ui_state.iter_mut().for_each(|block| match block {
                UiBlock::Salt { value, definition } => *value = gen_salt(definition, &mut self.rng),
                UiBlock::Category { .. } => (),
            });
        }
    }

    fn inc_file_index_by(&self, n: usize, current: usize) -> usize {
        (current + n) % self.files.len()
    }

    fn dec_file_index_by(&self, n: usize, current: usize) -> usize {
        (current as isize - n as isize).rem_euclid(self.files.len() as isize) as usize
    }

    fn mk_filename(&mut self) -> StdResult<String, String> {
        match filename::selection_to_filename(&self.schema, &self.ui_state) {
            Ok(name) => {
                let ext = match self.active_file().extension() {
                    Some(ext) => format!(".{}", ext.to_string_lossy()),
                    None => String::new(),
                };
                Ok(format!("{name}{ext}"))
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

    fn load_active(&mut self) -> egui::Image {
        let uri = Self::to_uri(self.active_file());
        // skip the io if this uri is already in the cache
        if self.ctx.try_load_bytes(&uri).is_ok() {
            return egui::Image::from_uri(uri);
        }

        match File::open(self.active_file()) {
            Err(e) => {
                error!("{e}");
                // skip this file so the rest can still be worked with
                self.files.remove(self.active);
                // load the next one instead
                self.load_active()
            }
            Ok(mut file) => {
                let mut buffer = vec![];
                match file.read_to_end(&mut buffer) {
                    Err(e) => {
                        error!("{e}");
                        // skip this file so the rest can still be worked with
                        self.files.remove(self.active);
                        // load the next one instead
                        self.load_active()
                    }
                    Ok(_) => {
                        self.ctx.include_bytes(uri.clone(), buffer);
                        egui::Image::from_uri(uri)
                    }
                }
            }
        }
    }

    /// sets the ui_state if the current file's filename can be parsed
    /// returns true if it was successful, false it set the state to the empty state
    fn parse_current_file(&mut self) -> bool {
        let parsed = self
            .schema
            .parse(&self.active_file().file_stem().unwrap().to_string_lossy());
        let success = parsed.is_ok();
        let state = parsed.unwrap_or_else(|_| to_empty_state(&self.schema, &mut self.rng));
        self.ui_state = state;
        success
    }

    fn apply_rename(&mut self) {
        // only apply the rename if there isn't an error generating the new filename
        if let Ok(filename) = self.mk_filename() {
            let mut to = self.working_dir.clone();
            to.push(&filename);
            match std::fs::rename(self.active_file(), &to) {
                Ok(()) => info!(
                    "{} →  {}",
                    self.active_file()
                        .file_name()
                        .map_or("old".into(), |os| os.to_string_lossy()),
                    filename
                ),
                Err(e) => error!("{}", Error::FailedRename(e)),
            };

            // the image will never be refrenced by its old name again so evict it from the cache
            self.ctx.forget_image(&Self::to_uri(self.active_file()));

            // update the list of filenames so the next refresh doesn't fail
            self.files[self.active] = to;
        }
    }
}

// rng generates the first salt
pub fn to_empty_state(schema: &Schema, rng: &mut ChaCha8Rng) -> Vec<UiBlock> {
    schema
        .blocks()
        .iter()
        .map(|block| match block {
            schema::Block::Salt(salt) => UiBlock::Salt {
                value: gen_salt(salt, rng),
                definition: salt.clone(),
            },
            schema::Block::Category(cat) => {
                let values = cat
                    .values()
                    .iter()
                    .map(|name| (name.clone(), false))
                    .collect();
                UiBlock::Category {
                    name: cat.name().to_string(),
                    values,
                }
            }
        })
        .collect()
}

impl eframe::App for App {
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

                for block in &mut self.ui_state {
                    match block {
                        UiBlock::Category { name, values } => {
                            ui.label(name.clone());
                            for (name, checked) in values {
                                let label = format!(
                                    "{name} ({})",
                                    self.parsed_counts.get(name).unwrap_or(&0)
                                );
                                ui.checkbox(checked, &label);
                            }
                        }
                        UiBlock::Salt { .. } => (),
                    }
                }
            });
        });

        egui::TopBottomPanel::new(TopBottomSide::Top, "filename").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new("filename:"));

                let filename = self
                    .active_file()
                    .file_name()
                    // filename errors should be handled by app logic. Just display an empty string till the app catches up.
                    .map_or(String::new(), |fname| fname.to_string_lossy().to_string());

                ui.add(Label::new(&filename));

                let open_button = ui
                    .add(Button::new("Open"))
                    .on_hover_text("Open in the default app");

                if open_button.clicked() {
                    if let Err(e) =
                        open::that_detached(self.active_file()).map_err(Error::FailedToOpen)
                    {
                        error!("{e}");
                        let url = format!(
                            "file://{}/{}",
                            // filename errors should be handled by app logic. Just display an empty string till the app catches up.
                            self.working_dir.to_str().unwrap_or(""),
                            &filename
                        );
                        // attempt to open in a browser instead ignoring failures
                        let _ = open::that_detached(url);
                    }
                }
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
                let zoom = self.zoom;
                let image = self
                    .load_active()
                    .rounding(10.0)
                    .fit_to_fraction(egui::Vec2 { x: zoom, y: zoom });

                ui.add(image);
            });
        });
    }
}
