//! Grid layout widget for Cadiotheka.

use crate::components::{Card, CardAction, CardData};
use crate::i18n;

/// State and rendering for a reusable grid component.
#[derive(Default)]
pub struct Grid;

impl Grid {
    /// Compute the number of columns and the width of each card for a given
    /// available width, using the same constants as the rendered grid.
    pub fn column_metrics(available_width: f32) -> (usize, f32) {
        let inner_spacing = 20.0;
        let min_card_width = 240.0;
        let columns = ((available_width + inner_spacing) / (min_card_width + inner_spacing))
            .floor()
            .clamp(1.0, 6.0) as usize;
        let card_width =
            (available_width - inner_spacing * (columns as f32 - 1.0)) / columns as f32;
        (columns, card_width)
    }

    /// Draw a grid of cards, or an empty-state message if there are none.
    ///
    /// Returns any actions triggered by clicking interactive elements.
    pub fn show(&self, ui: &mut egui::Ui, cards: &[CardData]) -> Vec<CardAction> {
        let margin = 24.0;
        let inner_spacing = 20.0;
        let mut actions = Vec::new();

        if cards.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(64.0);
                ui.label(
                    egui::RichText::new(i18n::Grid::EMPTY_TITLE)
                        .heading()
                        .size(24.0),
                );
                ui.add_space(8.0);
                ui.label(i18n::Grid::EMPTY_MESSAGE);
                ui.add_space(16.0);
                if ui.button(i18n::Grid::CLEAR_SEARCH).clicked() {
                    actions.push(CardAction::ClearSearch);
                }
            });
            return actions;
        }

        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: margin as i8,
                right: margin as i8,
                top: 0,
                bottom: margin as i8,
            })
            .show(ui, |ui| {
                let available_width = ui.available_width();
                let (columns, card_width) = Self::column_metrics(available_width);

                let card = Card;

                for (row_idx, row) in cards.chunks(columns).enumerate() {
                    ui.horizontal(|ui| {
                        for (col, data) in row.iter().enumerate() {
                            let index = row_idx * columns + col;
                            ui.vertical(|ui| {
                                ui.set_min_width(card_width);
                                ui.set_max_width(card_width);
                                if let Some(action) = card.show(ui, index, data) {
                                    actions.push(action);
                                }
                            });
                            if col + 1 < row.len() {
                                ui.add_space(inner_spacing);
                            }
                        }
                    });
                    ui.add_space(inner_spacing);
                }
            });

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_metrics_clamps_to_one_for_narrow_widths() {
        let (columns, width) = Grid::column_metrics(100.0);
        assert_eq!(columns, 1);
        assert_eq!(width, 100.0);
    }

    #[test]
    fn column_metrics_fits_two_columns_at_exact_breakpoint() {
        // (width + spacing) / (min + spacing) == 2
        let min_card_width = 240.0;
        let inner_spacing = 20.0;
        let breakpoint = 2.0 * (min_card_width + inner_spacing) - inner_spacing;
        let (columns, width) = Grid::column_metrics(breakpoint);
        assert_eq!(columns, 2);
        assert!((width - min_card_width).abs() < f32::EPSILON);
    }

    #[test]
    fn column_metrics_caps_at_six_columns() {
        let (columns, width) = Grid::column_metrics(10_000.0);
        assert_eq!(columns, 6);
        let expected_width = (10_000.0 - 20.0 * 5.0) / 6.0;
        assert!((width - expected_width).abs() < 0.01);
    }

    #[test]
    fn column_metrics_increases_columns_with_width() {
        let (narrow, _) = Grid::column_metrics(200.0);
        let (medium, _) = Grid::column_metrics(600.0);
        let (wide, _) = Grid::column_metrics(1_400.0);

        assert_eq!(narrow, 1);
        assert!(medium >= 2);
        assert!(wide >= 5);
    }
}
