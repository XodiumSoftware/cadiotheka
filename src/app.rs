//! The main Cadiotheka hub application state and UI.

use crate::components::{Footer, Header, View};
use crate::pages::Hub;
use crate::theme;
use egui_phosphor_icons::add_fonts;

/// The Cadiotheka hub application.
#[derive(Default)]
pub struct CadiothekaApp {
    /// Main hub UI.
    hub: Hub,
    /// Top navigation header.
    header: Header,
    /// Bottom navigation footer.
    footer: Footer,
}

impl CadiothekaApp {
    /// Creates the app and registers the Phosphor icon fonts and image loaders.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_style_of(egui::Theme::Dark, theme::style());
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);

        let mut fonts = egui::FontDefinitions::default();
        add_fonts(&mut fonts);
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("phosphor-icons".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self::default()
    }
}

impl eframe::App for CadiothekaApp {
    /// Renders the hub UI each frame.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.header.show(ui);
        self.footer.show(ui);
        egui::CentralPanel::default().show(ui, |ui| match self.header.view() {
            View::Hub => self.hub.show(ui),
        });
    }
}
