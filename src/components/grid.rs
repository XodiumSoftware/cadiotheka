//! Grid layout widget for Cadiotheka.

use crate::components::{Card, CardData};

/// State and rendering for a reusable grid component.
#[derive(Default)]
pub struct Grid;

impl Grid {
    /// Draw a grid of cards.
    pub fn show(&self, ui: &mut egui::Ui, cards: &[CardData]) {
        let margin = 24.0;
        let inner_spacing = 20.0;

        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: margin as i8,
                right: margin as i8,
                top: 0,
                bottom: margin as i8,
            })
            .show(ui, |ui| {
                let available_width = ui.available_width();
                let min_card_width = 240.0;
                let columns = ((available_width + inner_spacing) / (min_card_width + inner_spacing))
                    .floor()
                    .clamp(1.0, 6.0) as usize;
                let card_width =
                    (available_width - inner_spacing * (columns as f32 - 1.0)) / columns as f32;

                let card = Card;

                for row in cards.chunks(columns) {
                    ui.horizontal(|ui| {
                        for (i, data) in row.iter().enumerate() {
                            ui.vertical(|ui| {
                                ui.set_min_width(card_width);
                                ui.set_max_width(card_width);
                                card.show(ui, data);
                            });
                            if i + 1 < row.len() {
                                ui.add_space(inner_spacing);
                            }
                        }
                    });
                    ui.add_space(inner_spacing);
                }
            });
    }
}
