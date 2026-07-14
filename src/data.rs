//! Static data fixtures for Cadiotheka.

use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use serde::{Deserialize, Serialize};

/// A URL pointing to a card's icon asset.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct IconUrl(pub String);

/// Data displayed on a content card.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CardData {
    /// Card title.
    pub title: String,
    /// Author or creator name.
    pub author: String,
    /// Short description of the content.
    pub description: String,
    /// Extended markdown description shown in the project detail modal.
    #[serde(default)]
    pub extended_desc: String,
    /// Categorized tags for the content.
    pub tags: Vec<Tag>,
    /// Supported platforms for the content.
    pub supported_platforms: Vec<Platform>,
    /// Download count.
    pub downloads: u64,
    /// Favorite count.
    pub favorites: u64,
    /// Official timestamp for when the card was published or updated.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
    /// Optional icon URL (when absent, a colored placeholder is generated).
    pub icon_url: Option<IconUrl>,
}

/// Top-level fixture container.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Fixture {
    /// All cards in the fixture.
    pub cards: Vec<CardData>,
}

/// Load the embedded card fixture.
pub fn load_cards() -> Vec<CardData> {
    let fixture: Fixture = serde_json::from_str(include_str!("../test_data/cards.json"))
        .expect("cards fixture is valid JSON");
    fixture.cards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_url_serializes_transparently() {
        let url = IconUrl("https://example.com/icon.svg".to_owned());
        let json = serde_json::to_string(&url).unwrap();
        assert_eq!(json, "\"https://example.com/icon.svg\"");

        let decoded: IconUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, url);
    }

    #[test]
    fn load_cards_returns_entries() {
        let cards = load_cards();
        assert!(
            !cards.is_empty(),
            "fixture should contain at least one card"
        );
    }
}
