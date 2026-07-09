//! Main hub UI component shown after a successful login.

use crate::utils::Utils;

/// State for the hub UI.
#[derive(Default)]
pub struct Hub;

impl Hub {
    /// Renders the hub UI.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        Utils::dotted_background(ui);
    }
}
