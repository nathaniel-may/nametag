pub mod app;
pub mod config;
pub mod error;
pub mod filename;
pub mod fs_util;
pub mod parse_state_machine;
pub mod schema;
pub mod util;

use app::{App, AppIcons, AppView};
use eframe::egui::{self, include_image, FontFamily};
use error::{Error, Result};
use std::sync::Arc;
use tracing::info;

pub fn run() -> Result<()> {
    // set up logging
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_line_number(false)
        .with_thread_ids(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(Error::LoggerFailed)?;

    // create and run the app
    let mut app = App {
        // dummy ctx that gets immediately overwritten.
        ctx: Arc::new(egui::Context::default()),
        view: AppView::DirSelect,
        icons: AppIcons {
            folder: include_image!("../assets/icons/folder.png"),
            tag: include_image!("../assets/icons/tag.png"),
            query: include_image!("../assets/icons/search.png"),
        },
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
