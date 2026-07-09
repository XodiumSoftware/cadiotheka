//! Main hub UI component shown after a successful login.

use crate::components::Footer;
use crate::i18n;

/// State for the hub UI.
#[derive(Default)]
pub struct Hub {
    /// Name entered by the user.
    name: String,
    /// Counter value.
    counter: i32,
    /// Footer widget.
    footer: Footer,
}

impl Hub {
    /// Renders the hub UI.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(i18n::Hub::HEADING);
            ui.horizontal(|ui| {
                ui.label(i18n::Hub::NAME_LABEL);
                ui.text_edit_singleline(&mut self.name);
            });
            ui.horizontal(|ui| {
                ui.label(i18n::Hub::COUNTER_LABEL);
                if ui.button(i18n::Hub::DECREMENT_BUTTON).clicked() {
                    self.counter -= 1;
                }
                ui.label(self.counter.to_string());
                if ui.button(i18n::Hub::INCREMENT_BUTTON).clicked() {
                    self.counter += 1;
                }
            });
        });

        self.footer.show(ui);
    }
}
