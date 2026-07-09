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
                ui.heading(egui::RichText::new(i18n::Hub::HEADER).strong());
                ui.separator();
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.selectable_value(
                        &mut self.view,
                        View::Hub,
                        format!("{} {}", i18n::Hub::HUB_ICON, i18n::Hub::HUB_BUTTON),
                    );
                });
            });
        });
    }
}
