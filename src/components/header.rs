//! Top navigation header for Cadiotheka.

use crate::i18n;

/// The currently selected main view.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum View {
    /// The main hub dashboard.
    #[default]
    Hub,
}

/// Top navigation header for the main application window.
#[derive(Default)]
pub struct Header {
    /// Currently selected view in the hub.
    pub view: View,
}

impl Header {
    /// Draw the top navigation header.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("hub_header").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(i18n::Hub::HEADER);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.view, View::Hub, i18n::Hub::HUB_BUTTON);
                });
            });
        });
    }
}
