//! Main hub UI component shown after a successful login.

use crate::components::card::IconUrl;
use crate::components::{CardData, DottedBackground, Grid, Keycap, SearchBar};
use crate::engines::SearchEngine;
use crate::platforms::Platform;
use crate::tags::Tag;

/// Container shape for cards loaded from `test_data/cards.json`.
#[derive(Debug, serde::Deserialize)]
struct CardFile {
    cards: Vec<CardEntry>,
}

/// A single card entry as stored in the JSON fixture.
#[derive(Debug, serde::Deserialize)]
struct CardEntry {
    title: String,
    author: String,
    description: String,
    tags: Vec<Tag>,
    supported_platforms: Vec<Platform>,
    downloads: u64,
    favorites: u64,
    #[serde(with = "time::serde::rfc3339")]
    timestamp: time::OffsetDateTime,
    icon_url: Option<IconUrl>,
}

impl CardEntry {
    /// Convert this entry into an owned [`CardData`].
    fn into_card_data(self) -> CardData {
        CardData {
            title: self.title,
            author: self.author,
            description: self.description,
            tags: self.tags,
            supported_platforms: self.supported_platforms,
            downloads: self.downloads,
            favorites: self.favorites,
            timestamp: self.timestamp,
            icon_url: self.icon_url,
        }
    }
}

/// State for the hub UI.
pub struct Hub {
    /// Search engine owning the loaded cards.
    engine: SearchEngine,
    /// Search control state.
    search_bar: SearchBar,
}

impl Default for Hub {
    fn default() -> Self {
        let fixture = include_str!("../../test_data/cards.json");
        let file: CardFile = serde_json::from_str(fixture)
            .expect("test_data/cards.json should contain valid card fixtures");

        Self {
            engine: SearchEngine::new(
                file.cards
                    .into_iter()
                    .map(CardEntry::into_card_data)
                    .collect(),
            ),
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
