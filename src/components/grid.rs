//! Grid layout widget for Cadiotheka.

use crate::components::builders::Keycap;
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
                let button_text = i18n::Grid::CLEAR_SEARCH;
                let button_font = ui
                    .style()
                    .text_styles
                    .get(&egui::TextStyle::Button)
                    .cloned()
                    .unwrap_or_else(|| egui::FontId::proportional(14.0));
                let or_font = ui
                    .style()
                    .text_styles
                    .get(&egui::TextStyle::Body)
                    .cloned()
                    .unwrap_or_else(|| egui::FontId::proportional(14.0));
                let keycap_font = egui::FontId::proportional(12.0);

                let button_galley = ui.ctx().fonts_mut(|f| {
                    f.layout(
                        button_text.to_owned(),
                        button_font,
                        ui.visuals().text_color(),
                        f32::INFINITY,
                    )
                });
                let or_galley = ui.ctx().fonts_mut(|f| {
                    f.layout(
                        "or".to_owned(),
                        or_font,
                        ui.visuals().text_color(),
                        f32::INFINITY,
                    )
                });
                let keycap_galley = ui.ctx().fonts_mut(|f| {
                    f.layout(
                        "Ctrl + C".to_owned(),
                        keycap_font,
                        ui.visuals().text_color(),
                        f32::INFINITY,
                    )
                });

                let button_padding = ui.spacing().button_padding;
                let button_size = button_galley.size() + 2.0 * button_padding;
                let or_size = or_galley.size();
                let keycap_size = egui::vec2(keycap_galley.size().x + 10.0, 20.0);
                let gap = 8.0;
                let total_width = button_size.x + gap + or_size.x + gap + keycap_size.x;

                ui.horizontal(|ui| {
                    let left = (ui.available_width() - total_width).max(0.0) / 2.0;
                    ui.add_space(left);
                    if clear_search_button(ui).clicked() {
                        actions.push(CardAction::ClearSearch);
                    }
                    ui.label("or");
                    Keycap::builder()
                        .keys(&[egui::Key::C])
                        .ctrl(true)
                        .combine(true)
                        .inline(true)
                        .build(ui);
                });
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

/// Draws a plain "Clear search" button.
fn clear_search_button(ui: &mut egui::Ui) -> egui::Response {
    ui.button(i18n::Grid::CLEAR_SEARCH)
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
