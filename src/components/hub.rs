//! Main hub UI component shown after a successful login.

use crate::i18n;

/// State for the main hub demo UI.
#[derive(Default)]
pub struct Hub {
    /// Name entered by the user in the demo UI.
    name: String,
    /// Demo counter value.
    counter: i32,
}

impl Hub {
    /// Renders the hub demo UI.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading(i18n::Demo::HEADING);
        ui.horizontal(|ui| {
            ui.label(i18n::Demo::NAME_LABEL);
            ui.text_edit_singleline(&mut self.name);
        });
        ui.horizontal(|ui| {
            ui.label(i18n::Demo::COUNTER_LABEL);
            if ui.button(i18n::Demo::DECREMENT_BUTTON).clicked() {
                self.counter -= 1;
            }
            ui.label(self.counter.to_string());
            if ui.button(i18n::Demo::INCREMENT_BUTTON).clicked() {
                self.counter += 1;
            }
        });
    }
}
