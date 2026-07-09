//! The main Cadiotheka hub application state and UI.

use crate::i18n;

/// The Cadiotheka hub application.
#[derive(Default)]
pub struct CadiothekaApp {
    /// Name entered by the user in the demo UI.
    name: String,
    /// Demo counter value.
    counter: i32,
}

impl eframe::App for CadiothekaApp {
    /// Renders the hub UI each frame.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(i18n::HEADING);
            ui.horizontal(|ui| {
                ui.label(i18n::NAME_LABEL);
                ui.text_edit_singleline(&mut self.name);
            });
            ui.horizontal(|ui| {
                ui.label(i18n::COUNTER_LABEL);
                if ui.button(i18n::DECREMENT_BUTTON).clicked() {
                    self.counter -= 1;
                }
                ui.label(self.counter.to_string());
                if ui.button(i18n::INCREMENT_BUTTON).clicked() {
                    self.counter += 1;
                }
            });
        });
    }
}
