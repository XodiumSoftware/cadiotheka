//! Main hub UI component shown after a successful login.
//!
//! The card catalog is currently loaded from the embedded fixture managed by
//! [`crate::fixture`]. See that module for notes on the planned move to a
//! runtime data source.

use crate::components::{DottedBackground, Grid, Keycap, SearchBar};
use crate::engines::SearchEngine;
use crate::fixture::load_cards;

/// State for the hub UI.
pub struct Hub {
    /// Search engine owning the loaded cards.
    engine: SearchEngine,
    /// Search control state.
    search_bar: SearchBar,
}

impl Default for Hub {
    fn default() -> Self {
        let cards = load_cards().expect("card fixture should be valid and non-empty");

        Self {
            engine: SearchEngine::new(cards),
            search_bar: SearchBar::default(),
        }
    }
}

impl Hub {
    /// Renders the hub UI.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        DottedBackground::builder()
            .spacing(24.0)
            .radius(1.0)
            .base_alpha(0.4)
            .fade_start(0.75)
            .build(ui);

        ui.add_space(24.0);
        let inner_spacing = 20.0;
        let (columns, card_width) = Grid::column_metrics(ui.available_width());
        let search_width = if columns >= 2 {
            card_width * 2.0 + inner_spacing
        } else {
            card_width
        };

        let suggestions = self.engine.suggestions();
        let parsed = ui
            .vertical_centered(|ui| {
                ui.set_max_width(search_width);
                self.search_bar
                    .show(ui, &suggestions, SearchEngine::parse_query)
            })
            .inner;

        let mut focus_search = false;
        Keycap::builder()
            .keys(&[egui::Key::C, egui::Key::S])
            .execute(|| focus_search = true)
            .build(ui);

        if focus_search {
            ui.memory_mut(|mem| mem.request_focus(self.search_bar.id));
        }

        ui.add_space(24.0);
        let cards = self.engine.search(&parsed);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .content_margin(egui::Margin {
                left: 0,
                right: 48,
                top: 0,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.add_space(16.0);
                Grid.show(ui, &cards);
            });
    }
}
