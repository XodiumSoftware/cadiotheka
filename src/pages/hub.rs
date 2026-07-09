//! Main hub UI component shown after a successful login.

use crate::components::DottedBackground;

/// State for the hub UI.
#[derive(Default)]
pub struct Hub;

impl Hub {
    /// Renders the hub UI.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        DottedBackground::builder()
            .spacing(24.0)
            .radius(1.0)
            .base_alpha(0.4)
            .fade_start(0.75)
            .build(ui);
    }
}
