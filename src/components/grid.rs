//! Grid layout widget for Cadiotheka.

use crate::components::{Card, CardData};

/// State and rendering for a reusable grid component.
#[derive(Default)]
pub struct Grid;

impl Grid {
    /// Draw a grid of cards.
    pub fn show(&self, ui: &mut egui::Ui, cards: &[CardData]) {
        let available_width = ui.available_width();
        let min_card_width = 240.0;
        let spacing = ui.spacing().item_spacing.x;
        let columns = ((available_width + spacing) / (min_card_width + spacing))
            .floor()
            .clamp(1.0, 6.0) as usize;
        let card_width = (available_width - spacing * (columns as f32 - 1.0)) / columns as f32;

        let card = Card;

        for row in cards.chunks(columns) {
            ui.horizontal(|ui| {
                for data in row {
                    ui.vertical(|ui| {
                        ui.set_min_width(card_width);
                        ui.set_max_width(card_width);
                        card.show(ui, data);
                    });
                    if !std::ptr::eq(data, row.last().unwrap()) {
                        ui.add_space(spacing);
                    }
                }
            });
        }
    }
}
