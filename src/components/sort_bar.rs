//! Sort bar widget for the Cadiotheka hub.

/// State and rendering for a sort control bar.
#[derive(Default)]
pub struct SortBar;

impl SortBar {
    /// Draw the sort bar inside a card-like container.
    pub fn show(&self, ui: &mut egui::Ui) {
        let margin = 24.0;
        let mut frame = egui::Frame::group(ui.style());
        frame.fill = frame.fill.gamma_multiply(0.65);
        egui::Frame::new()
            .inner_margin(egui::Margin::same(margin as i8))
            .show(ui, |ui| {
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Sort by:").strong());
                        ui.label("Relevance");
                        ui.label("Downloads");
                        ui.label("Favorites");
                        ui.label("Newest");
                    });
                });
            });
    }
}
