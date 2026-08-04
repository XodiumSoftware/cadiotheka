//! Supported CAD platforms for Cadiotheka content.
//!
//! Platforms are stored in the backend D1 database (`schemas/platforms.sql`)
//! and served over `GET /data/platforms`. Each record pairs a stable wire id
//! with its user-facing label and Tailwind color class.

use serde::{Deserialize, Serialize};

/// A supported CAD platform record fetched from the backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Platform {
    /// Stable wire id stored on project rows (e.g. `fusion_360`).
    pub id: String,
    /// User-facing label (e.g. `Fusion 360`).
    pub label: String,
    /// Tailwind-compatible CSS color class.
    pub color: String,
}

/// Convenience accessor for a platform's user-facing label.
pub fn platform_label(platform: &Platform) -> &str {
    &platform.label
}

/// Convenience accessor for a platform's Tailwind color class.
pub fn platform_color(platform: &Platform) -> &str {
    &platform.color
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Platform {
        Platform {
            id: "fusion_360".to_owned(),
            label: "Fusion 360".to_owned(),
            color: "text-yellow-700".to_owned(),
        }
    }

    #[test]
    fn platform_roundtrips_json() {
        let platform = sample();
        let json = serde_json::to_string(&platform).unwrap();
        let decoded: Platform = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, platform);
    }

    #[test]
    fn platform_helpers_expose_fields() {
        let platform = sample();
        assert_eq!(platform_label(&platform), "Fusion 360");
        assert_eq!(platform_color(&platform), "text-yellow-700");
    }
}
