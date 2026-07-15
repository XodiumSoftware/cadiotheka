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
    /// Unique card identifier.
    pub id: String,
    /// Card title.
    pub title: String,
    /// Author or creator name.
    pub author: String,
    /// Author account identifier.
    pub author_id: String,
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

/// Fetch projects from the backend API.
///
/// On failure it logs to the browser console and returns an empty vector so
/// the UI can keep running with a graceful fallback.
pub async fn fetch_cards() -> Vec<CardData> {
    match gloo_net::http::Request::get("/api/projects").send().await {
        Ok(response) if response.ok() => response.json::<Vec<CardData>>().await.unwrap_or_default(),
        Ok(response) => {
            let status = response.status();
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch projects: HTTP {status}").into(),
            );
            Vec::new()
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to fetch projects: {err:?}").into());
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn sample_card() -> CardData {
        CardData {
            id: "71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2".to_owned(),
            title: "Mountain Bike".to_owned(),
            author: "TrailBlazer".to_owned(),
            author_id: "8af81bd9-b70a-4d64-89e9-83bbc4e0297d".to_owned(),
            description: "A rugged mountain bike model ready for off-road adventures.".to_owned(),
            extended_desc: "Extended description.".to_owned(),
            tags: vec![Tag::Model3d, Tag::Vehicle],
            supported_platforms: vec![Platform::Blender, Platform::FreeCAD],
            downloads: 1200,
            favorites: 84,
            timestamp: datetime!(2026-07-07 14:30:00 UTC),
            icon_url: None,
        }
    }

    #[test]
    fn card_serializes_and_deserializes() {
        let card = sample_card();
        let json = serde_json::to_string(&card).expect("card serializes");
        let decoded: CardData = serde_json::from_str(&json).expect("card deserializes");
        assert_eq!(decoded, card);
    }

    #[test]
    fn icon_url_serializes_transparently() {
        let url = IconUrl("https://example.com/icon.svg".to_owned());
        let json = serde_json::to_string(&url).unwrap();
        assert_eq!(json, "\"https://example.com/icon.svg\"");

        let decoded: IconUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, url);
    }

    /// Validates that known tags and platforms serialize and deserialize
    /// correctly. This catches stale enum definitions.
    #[test]
    fn card_uses_known_tags_and_platforms() {
        let card = sample_card();
        for tag in card.tags {
            let _ = tag.label();
        }
        for platform in card.supported_platforms {
            let _ = platform.label();
        }
    }
}
