//! Main hub UI component shown after a successful login.

use crate::components::card::IconUrl;
use crate::components::{CardData, DottedBackground, Grid, SortBar};
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
#[derive(Default)]
pub struct Hub;

impl Hub {
    /// Renders the hub UI.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        DottedBackground::builder()
            .spacing(24.0)
            .radius(1.0)
            .base_alpha(0.4)
            .fade_start(0.75)
            .build(ui);

        let fixture = include_str!("../../test_data/cards.json");
        let file: CardFile = serde_json::from_str(fixture)
            .expect("test_data/cards.json should contain valid card fixtures");

        let cards: Vec<CardData> = file
            .cards
            .into_iter()
            .map(CardEntry::into_card_data)
            .collect();

        egui::ScrollArea::vertical().show(ui, |ui| {
            SortBar.show(ui);
            ui.add_space(16.0);
            Grid.show(ui, &cards);
        });
    }
}
