//! Content tags and categories for Cadiotheka.
//!
//! Tags are stored in the backend D1 database (`schemas/tags.sql`) and served
//! over `GET /data/tags`. Each record pairs a stable wire id with its
//! user-facing label and an inline CSS color style string.

use serde::{Deserialize, Serialize};

/// A content tag record fetched from the backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Tag {
    /// Stable wire id stored on project rows (e.g. `3d_model`).
    pub id: String,
    /// User-facing label (e.g. `3D Model`).
    pub label: String,
    /// Inline CSS color style string, applied directly to the rendered element.
    pub color: String,
}

/// Convenience accessor for a tag's user-facing label.
pub fn tag_label(tag: &Tag) -> &str {
    &tag.label
}

/// Convenience accessor for a tag's inline CSS color style.
pub fn tag_color(tag: &Tag) -> &str {
    &tag.color
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tag {
        Tag {
            id: "3d_model".to_owned(),
            label: "3D Model".to_owned(),
            color: "background-color:#1d4ed8;color:#ffffff".to_owned(),
        }
    }

    #[test]
    fn tag_roundtrips_json() {
        let tag = sample();
        let json = serde_json::to_string(&tag).unwrap();
        let decoded: Tag = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn tag_helpers_expose_fields() {
        let tag = sample();
        assert_eq!(tag_label(&tag), "3D Model");
        assert_eq!(tag_color(&tag), "background-color:#1d4ed8;color:#ffffff");
    }
}
