//! Fixture loading and validation for card data.
//!
//! ⚠️ The card fixture is currently embedded at compile time via
//! [`include_str!`] (`test_data/cards.json`). This is a temporary setup for
//! development and offline demos. The long-term plan is to load the catalog
//! from a remote source at runtime — most likely a JSON endpoint or a
//! generated index (e.g., a GitHub repository index) — so the hub can stay
//! current without rebuilding the app.

use crate::components::card::{CardData, IconUrl};
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

/// The embedded card fixture.
pub const FIXTURE_JSON: &str = include_str!("../test_data/cards.json");

/// Load and validate the embedded card fixture.
///
/// Returns an error if the fixture is missing, malformed, or contains
/// unknown `Tag`/`Platform` variants. Because the fixture deserializes
/// directly into the enum types, any drift between `cards.json` and the
/// Rust definitions will be caught here.
pub fn load_cards() -> Result<Vec<CardData>, String> {
    let file: CardFile = serde_json::from_str(FIXTURE_JSON)
        .map_err(|e| format!("failed to parse test_data/cards.json: {e}"))?;

    if file.cards.is_empty() {
        return Err("test_data/cards.json contains no cards".to_owned());
    }

    Ok(file
        .cards
        .into_iter()
        .map(CardEntry::into_card_data)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_loads_and_matches_enums() {
        let cards = load_cards().expect("embedded fixture should be valid");
        assert!(
            !cards.is_empty(),
            "fixture should contain at least one card"
        );
    }
}
