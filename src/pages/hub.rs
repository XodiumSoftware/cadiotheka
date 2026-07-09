//! Main hub UI component shown after a successful login.

use crate::components::{CardData, DottedBackground, Grid};
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
    icon: Option<String>,
}

impl CardEntry {
    /// Borrow this entry as a [`CardData`] view.
    fn as_card_data(&self) -> CardData<'_> {
        CardData {
            title: self.title.as_str(),
            author: self.author.as_str(),
            description: self.description.as_str(),
            tags: self.tags.clone(),
            supported_platforms: self.supported_platforms.clone(),
            downloads: self.downloads,
            favorites: self.favorites,
            timestamp: self.timestamp,
            icon: self.icon.as_deref(),
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

        let cards: Vec<CardData> = file.cards.iter().map(CardEntry::as_card_data).collect();

        Grid.show(ui, &cards);
    }
}
