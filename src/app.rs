use crate::{
    config,
    error::{Error, Result},
    filename::{self, gen_salt},
    fs_util,
    schema::{self, Schema},
};
use eframe::egui::{
    self,
    panel::{Side, TopBottomSide},
    Align, Button, Color32, FontFamily, ImageButton, ImageSource, Key, Label, Layout,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::Arc,
};
use tracing::{error, info, warn};

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
pub enum AppView {
    DirSelect,
    ApplyTags(AppDir),
    // Query(AppDir)
}

#[derive(Clone, Debug)]
pub struct AppDir {
    pub path: PathBuf,
    pub ctx: Arc<egui::Context>,
    pub schema: Schema,
    pub active: usize,
    pub ui_state: Vec<UiBlock>,
    pub files: Vec<PathBuf>,
    pub parsed_counts: HashMap<String, usize>,
    pub rng: ChaCha8Rng,
    pub zoom: f32,
}

impl AppDir {
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

    /// returns the path to the active file
    fn active_file(&self) -> &Path {
        &self.files[self.active]
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

    fn clear_state(&mut self) {
        self.ui_state = to_empty_state(&self.schema, &mut self.rng);
    }

    fn load_active(&mut self) -> egui::Image {
        let uri = App::to_uri(self.active_file());
        // skip the io if this uri is already in the cache
        if self.ctx.try_load_bytes(&uri).is_ok() {
            return egui::Image::from_uri(uri);
        }

        match File::open(self.active_file()) {
            Err(e) => {
                warn!(
                    "Failed to open file {}. Originating error: {e}",
                    self.active_file().to_string_lossy()
                );
                // skip this file so the rest can still be worked with
                self.files.remove(self.active);
                // load the next one instead
                self.load_active()
            }
            Ok(mut file) => {
                let mut buffer = vec![];
                match file.read_to_end(&mut buffer) {
                    Err(e) => {
                        warn!(
                            "Failed to read file contents {}. Originating error: {e}",
                            self.active_file().to_string_lossy()
                        );
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

    fn apply_rename(&mut self) {
        // only apply the rename if there isn't an error generating the new filename
        if let Ok(filename) = self.mk_filename() {
            let mut to = self.path.clone();
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
            self.ctx.forget_image(&App::to_uri(self.active_file()));

            // update the list of filenames so the next refresh doesn't fail
            self.files[self.active] = to;
        }
    }

    fn delete_active(&mut self) {
        // remove from cache
        self.ctx.forget_image(&App::to_uri(self.active_file()));
        // remove from file system
        std::fs::remove_file(self.active_file()).unwrap();
        // remove from list of files
        self.files.remove(self.active);
    }
}

#[derive(Clone, Debug)]
pub struct AppIcons {
    pub folder: ImageSource<'static>,
    pub tag: ImageSource<'static>,
    pub query: ImageSource<'static>,
}

#[derive(Clone, Debug)]
pub struct App {
    pub ctx: Arc<egui::Context>,
    pub view: AppView,
    pub icons: AppIcons,
}

impl App {
    pub fn load_dir(&mut self, dir: PathBuf) -> Result<()> {
        info!("Loading working directory");
        let files: Vec<PathBuf> = fs_util::collect_filenames(&dir)?
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
        // UI expects to dispaly the first image. Exit loading if there's nothing in the directory.
        if files.is_empty() {
            return Err(Error::EmptyWorkingDir);
        }

        let mut schema_path = dir.clone();
        schema_path.push("schema.dhall");
        let contents = fs::read_to_string(schema_path).map_err(Error::FailedToReadContents)?;
        let input = config::parse_schema(&contents)?;
        let schema = schema::Schema::from_config(input)?;

        let mut rng = ChaCha8Rng::from_entropy();
        let ui_state = to_empty_state(&schema, &mut rng);
        let parsed_counts = App::reparse_recount(&schema, &files);

        self.view = AppView::ApplyTags(AppDir {
            path: dir,
            ctx: self.ctx.clone(),
            schema,
            active: 0,
            ui_state,
            files,
            parsed_counts,
            rng,
            zoom: 1.0,
        });
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

    fn to_uri(path: &Path) -> String {
        let mut uri = "bytes://".to_string();
        uri.push_str(&path.to_string_lossy());
        uri
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
        // menu is present for all app views
        egui::SidePanel::new(Side::Left, "menu")
            .exact_width(60.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(
                    Layout::top_down(Align::Center).with_cross_align(Align::Center),
                    |ui| {
                        let padding = 20.0;

                        let style = ui.style_mut();
                        style.override_font_id = Some(egui::FontId {
                            size: 13.0,
                            family: FontFamily::Proportional,
                        });

                        style.visuals.widgets.hovered.weak_bg_fill = Color32::GRAY;
                        style.visuals.widgets.hovered.bg_stroke.width = 0.0;
                        style.visuals.widgets.hovered.fg_stroke.width = 0.0;

                        style.visuals.widgets.active.weak_bg_fill = Color32::GRAY;
                        style.visuals.widgets.active.bg_stroke.width = 0.0;
                        style.visuals.widgets.active.fg_stroke.width = 0.0;

                        style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                        style.visuals.selection.stroke.width = 0.0;

                        let (open, mut tag, query) = (
                            ImageButton::new(self.icons.folder.clone())
                                .selected(false)
                                .tint(Color32::DARK_GRAY)
                                .rounding(5.0),
                            ImageButton::new(self.icons.tag.clone())
                                .selected(false)
                                .tint(Color32::DARK_GRAY)
                                .rounding(5.0),
                            ImageButton::new(self.icons.query.clone())
                                .selected(false)
                                .tint(Color32::DARK_GRAY)
                                .rounding(5.0),
                        );
                        match self.view {
                            AppView::DirSelect => (),
                            AppView::ApplyTags(_) => {
                                tag = tag.selected(true).tint(Color32::BLUE);
                            }
                        }

                        let open_img = ui.add(open);
                        let open_label = ui.add(Label::new("Open"));

                        ui.add_space(padding);

                        let tag_img = ui.add(tag);
                        let tag_label = ui.add(Label::new("Tag"));

                        ui.add_space(padding);

                        let query_img = ui.add(query);
                        let query_label = ui.add(Label::new("Query"));

                        if open_img.clicked() || open_label.clicked() {
                            // let the user pick a new folder
                            if let Some(new_path) = rfd::FileDialog::new().pick_folder() {
                                match &self.view {
                                    // check to see if it's the current folder. Don't reload if it is.
                                    AppView::ApplyTags(current) if current.path == new_path => (),
                                    _ => {
                                        // TODO handle these errors properly
                                        self.load_dir(new_path).unwrap();
                                    }
                                }
                            }
                        }

                        if tag_img.clicked() || tag_label.clicked() {
                            // TODO switches back to tagging if they're searching.
                        }

                        if query_img.clicked() || query_label.clicked() {
                            // TODO switches back to searching if they're tagging.
                        }
                    },
                );
            });

        // match on app view to decide on other panes and actions
        match &mut self.view {
            AppView::DirSelect => {
                egui::CentralPanel::default().show(ctx, |_| {});
            }
            AppView::ApplyTags(ad) => {
                if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
                    ad.prev();
                }

                if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                    ad.next();
                }

                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    ad.apply_rename()
                }

                if ctx.input(|i| i.key_pressed(Key::Backspace)) {
                    info!("delete pressed");
                    if let rfd::MessageDialogResult::Yes = rfd::MessageDialog::new()
                        .set_description("Delete this file?")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        ad.delete_active()
                    }
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
                                ad.clear_state();
                            }
                        });
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        for block in &mut ad.ui_state {
                            match block {
                                UiBlock::Category { name, values } => {
                                    ui.label(name.clone());
                                    for (name, checked) in values {
                                        let label = format!(
                                            "{name} ({})",
                                            ad.parsed_counts.get(name).unwrap_or(&0)
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

                        let filename = ad
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
                                open::that_detached(ad.active_file()).map_err(Error::FailedToOpen)
                            {
                                error!("{e}");
                                let url = format!(
                                    "file://{}/{}",
                                    // filename errors should be handled by app logic. Just display an empty string till the app catches up.
                                    ad.path.to_str().unwrap_or(""),
                                    &filename
                                );
                                // attempt to open in a browser instead ignoring failures
                                let _ = open::that_detached(url);
                            }
                        }
                    });

                    match ad.mk_filename() {
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
                    ad.zoom *= ctx.input(|i| i.zoom_delta());

                    egui::ScrollArea::both().show(ui, |ui| {
                        let zoom = ad.zoom;
                        let image = ad
                            .load_active()
                            .rounding(10.0)
                            .fit_to_fraction(egui::Vec2 { x: zoom, y: zoom });

                        ui.add(image);
                    });
                });
            }
        }
    }
}
