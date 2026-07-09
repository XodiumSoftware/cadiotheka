//! Grid layout widget for Cadiotheka.

use crate::components::{Card, CardData};

/// State and rendering for a reusable grid component.
#[derive(Default)]
pub struct Grid;

impl Grid {
    /// Draw a grid of cards.
    pub fn show(&self, ui: &mut egui::Ui, cards: &[CardData<'_>]) {
        let available_width = ui.available_width();
        let min_card_width = 220.0;
        let spacing = ui.spacing().item_spacing.x;
        let columns = ((available_width + spacing) / (min_card_width + spacing))
            .floor()
            .clamp(1.0, 8.0) as usize;
        let card_width = (available_width - spacing * (columns as f32 - 1.0)) / columns as f32;

        let card = Card;

        ui.horizontal_wrapped(|ui| {
            for data in cards {
                ui.add_space(spacing);
                ui.vertical(|ui| {
                    ui.set_min_width(card_width);
                    ui.set_max_width(card_width);
                    card.show(ui, data);
                });
            }
        });
    }
}
