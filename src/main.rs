//! Cadiotheka — the open hub for CAD creators.
//!
//! This crate provides a native desktop GUI built with [`egui`] and [`eframe`].

mod app;
mod i18n;

/// Entry point for the Cadiotheka hub.
fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title(i18n::WINDOW_TITLE),
        ..Default::default()
    };

    eframe::run_native(
        i18n::WINDOW_TITLE,
        options,
        Box::new(|_cc| Ok(Box::new(app::CadiothekaApp::default()))),
    )
    .expect(i18n::STARTUP_ERROR);
}
