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
                if clear_search_button(ui).clicked() {
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

/// Draws the "Clear search" button with a `Ctrl + C` keycap inside it.
fn clear_search_button(ui: &mut egui::Ui) -> egui::Response {
    let label_text = i18n::Grid::CLEAR_SEARCH;
    let keycap_label = "Ctrl + C";

    let label_font = egui::FontId::proportional(13.0);
    let keycap_font = egui::FontId::proportional(10.0);
    let keycap_visuals = ui.visuals().widgets.inactive;

    let label_galley = ui.ctx().fonts_mut(|f| {
        f.layout(
            label_text.to_owned(),
            label_font.clone(),
            ui.visuals().text_color(),
            f32::INFINITY,
        )
    });
    let keycap_galley = ui.ctx().fonts_mut(|f| {
        f.layout(
            keycap_label.to_owned(),
            keycap_font.clone(),
            keycap_visuals.fg_stroke.color,
            f32::INFINITY,
        )
    });

    let label_size = label_galley.size();
    let keycap_size = egui::vec2(keycap_galley.size().x + 8.0, 16.0);
    let content_spacing = 6.0;
    let content_size = egui::vec2(
        label_size.x + content_spacing + keycap_size.x,
        label_size.y.max(keycap_size.y),
    );
    let button_padding = ui.spacing().button_padding;
    let desired_size = content_size + 2.0 * button_padding;

    let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());
    let visuals = ui.style().interact(&response);

    ui.painter()
        .rect_filled(rect, visuals.corner_radius, visuals.bg_fill);
    ui.painter().rect_stroke(
        rect,
        visuals.corner_radius,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let content_rect = rect.shrink2(button_padding);
    let label_pos = egui::pos2(
        content_rect.left(),
        content_rect.center().y - label_size.y / 2.0,
    );
    ui.painter()
        .galley(label_pos, label_galley, visuals.fg_stroke.color);

    let keycap_rect = egui::Rect::from_min_size(
        egui::pos2(
            label_pos.x + label_size.x + content_spacing,
            content_rect.center().y - keycap_size.y / 2.0,
        ),
        keycap_size,
    );
    ui.painter()
        .rect_filled(keycap_rect, 3.0, keycap_visuals.bg_fill);
    ui.painter().rect_stroke(
        keycap_rect,
        3.0,
        keycap_visuals.fg_stroke,
        egui::StrokeKind::Inside,
    );
    let keycap_text_pos = keycap_rect.center() - keycap_galley.size() * 0.5;
    ui.painter().galley(
        keycap_text_pos,
        keycap_galley,
        keycap_visuals.fg_stroke.color,
    );

    response
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
