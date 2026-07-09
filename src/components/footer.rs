//! Footer widget for Cadiotheka.

use crate::i18n;

/// Footer state and rendering for the main application window.
#[derive(Default)]
pub struct Footer;

impl Footer {
    /// Draw the footer in a bottom panel.
    pub fn show(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom("hub_footer").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(i18n::Footer::COPYRIGHT_PREFIX);
                ui.hyperlink_to(i18n::Footer::COPYRIGHT_OWNER, i18n::Footer::COPYRIGHT_URL);
                ui.label(i18n::Footer::COPYRIGHT_SUFFIX);
                ui.label(i18n::Footer::POWERED_BY_PREFIX);
                ui.hyperlink_to(i18n::Footer::EGUI_LABEL, i18n::Footer::EGUI_URL);
                ui.label(i18n::Footer::AND);
                ui.hyperlink_to(i18n::Footer::EFRAME_LABEL, i18n::Footer::EFRAME_URL);
                ui.label(i18n::Footer::PERIOD);
            });
        });
    }
}
