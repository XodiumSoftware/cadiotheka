//! Footer widget for Cadiotheka.

/// Footer state and rendering for the main application window.
#[derive(Default)]
pub struct Footer;

impl Footer {
    /// Draw the footer in a bottom panel.
    pub fn show(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom("hub_footer").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        });
    }
}
