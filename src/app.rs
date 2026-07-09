//! The main Cadiotheka hub application state and UI.

use crate::components::{Footer, Header, View};
use crate::pages::Hub;

/// The Cadiotheka hub application.
#[derive(Default)]
pub struct CadiothekaApp {
    /// Main hub UI.
    hub: Hub,
    /// Top navigation header.
    header: Header,
    /// Bottom navigation footer.
    footer: Footer,
    /// Currently selected view in the hub.
    view: View,
}

impl eframe::App for CadiothekaApp {
    /// Renders the hub UI each frame.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.header.show(ui);
        self.view = self.header.view;
        self.footer.show(ui);
        egui::CentralPanel::default().show(ui, |ui| match self.view {
            View::Hub => self.hub.show(ui),
        });
    }
}
