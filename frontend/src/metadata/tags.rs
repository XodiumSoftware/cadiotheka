//! Hardcoded content tags for Cadiotheka.
//!
//! Tags are no longer stored in the D1 database; they are defined as enum
//! variants in this file. Project rows still store tag wire ids as JSON arrays,
//! so the frontend resolves labels and colors through the enum helpers below.

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

/// Content tag variants with stable wire ids, labels, and Tailwind color styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tag {
    /// Three-dimensional models and assets.
    #[serde(rename = "3d_model")]
    ThreeDModel,
    /// Two-dimensional drawings and diagrams.
    #[serde(rename = "2d_drawing")]
    TwoDDrawing,
    /// Parametric or algorithmically defined designs.
    Parametric,
    /// Designs intended for fabrication or manufacturing.
    Fabrication,
    /// Robotics parts, assemblies, and accessories.
    Robotics,
    /// Furniture designs.
    Furniture,
    /// Vehicles and vehicle parts.
    Vehicle,
    /// Architectural models and elements.
    Architecture,
    /// Electronics enclosures and components.
    Electronics,
    /// Tools, jigs, and workshop helpers.
    Tooling,
    /// Lighting fixtures and designs.
    Lighting,
    /// Do-it-yourself projects and hacks.
    Diy,
    /// Interior design objects and layouts.
    Interior,
    /// General engineering models.
    Engineering,
    /// Aerospace parts and assemblies.
    Aerospace,
    /// Decorative objects.
    Decor,
    /// Medical devices and helpers.
    Medical,
    /// Assets for games and real-time rendering.
    GameAsset,
    /// Artistic or sculptural models.
    Art,
    /// Educational models and demonstrations.
    Educational,
    /// Work-in-progress designs.
    Wip,
}

impl Tag {
    /// Stable wire id stored on project rows.
    pub fn id(&self) -> &'static str {
        match self {
            Tag::ThreeDModel => "3d_model",
            Tag::TwoDDrawing => "2d_drawing",
            Tag::Parametric => "parametric",
            Tag::Fabrication => "fabrication",
            Tag::Robotics => "robotics",
            Tag::Furniture => "furniture",
            Tag::Vehicle => "vehicle",
            Tag::Architecture => "architecture",
            Tag::Electronics => "electronics",
            Tag::Tooling => "tooling",
            Tag::Lighting => "lighting",
            Tag::Diy => "diy",
            Tag::Interior => "interior",
            Tag::Engineering => "engineering",
            Tag::Aerospace => "aerospace",
            Tag::Decor => "decor",
            Tag::Medical => "medical",
            Tag::GameAsset => "game_asset",
            Tag::Art => "art",
            Tag::Educational => "educational",
            Tag::Wip => "wip",
        }
    }

    /// User-facing label.
    pub fn label(&self) -> &'static str {
        match self {
            Tag::ThreeDModel => "3D Model",
            Tag::TwoDDrawing => "2D Drawing",
            Tag::Parametric => "Parametric",
            Tag::Fabrication => "Fabrication",
            Tag::Robotics => "Robotics",
            Tag::Furniture => "Furniture",
            Tag::Vehicle => "Vehicle",
            Tag::Architecture => "Architecture",
            Tag::Electronics => "Electronics",
            Tag::Tooling => "Tooling",
            Tag::Lighting => "Lighting",
            Tag::Diy => "DIY",
            Tag::Interior => "Interior",
            Tag::Engineering => "Engineering",
            Tag::Aerospace => "Aerospace",
            Tag::Decor => "Decor",
            Tag::Medical => "Medical",
            Tag::GameAsset => "Game Asset",
            Tag::Art => "Art",
            Tag::Educational => "Educational",
            Tag::Wip => "WIP",
        }
    }

    /// Tailwind inline color style string applied to rendered badges.
    pub fn color(&self) -> &'static str {
        match self {
            Tag::ThreeDModel => "background-color:#1d4ed8;color:#ffffff",
            Tag::TwoDDrawing => "background-color:#0e7490;color:#ffffff",
            Tag::Parametric => "background-color:#7e22ce;color:#ffffff",
            Tag::Fabrication => "background-color:#c2410c;color:#ffffff",
            Tag::Robotics => "background-color:#b91c1c;color:#ffffff",
            Tag::Furniture => "background-color:#92400e;color:#ffffff",
            Tag::Vehicle => "background-color:#15803d;color:#ffffff",
            Tag::Architecture => "background-color:#374151;color:#ffffff",
            Tag::Electronics => "background-color:#ca8a04;color:#ffffff",
            Tag::Tooling => "background-color:#475569;color:#ffffff",
            Tag::Lighting => "background-color:#d97706;color:#ffffff",
            Tag::Diy => "background-color:#ea580c;color:#ffffff",
            Tag::Interior => "background-color:#be123c;color:#ffffff",
            Tag::Engineering => "background-color:#334155;color:#ffffff",
            Tag::Aerospace => "background-color:#0369a1;color:#ffffff",
            Tag::Decor => "background-color:#e11d48;color:#ffffff",
            Tag::Medical => "background-color:#047857;color:#ffffff",
            Tag::GameAsset => "background-color:#be185d;color:#ffffff",
            Tag::Art => "background-color:#a21caf;color:#ffffff",
            Tag::Educational => "background-color:#0f766e;color:#ffffff",
            Tag::Wip => "background-color:#4d7c0f;color:#ffffff",
        }
    }

    /// All defined tags in display order.
    pub fn all() -> Vec<Tag> {
        Tag::iter().collect()
    }

    /// Resolves a tag from its wire id, or `None` if the id is unknown.
    pub fn from_id(id: &str) -> Option<Tag> {
        Tag::iter().find(|tag| tag.id() == id)
    }
}

/// Convenience accessor for a tag's user-facing label.
pub fn tag_label(tag: &Tag) -> &'static str {
    tag.label()
}

/// Convenience accessor for a tag's inline CSS color style.
pub fn tag_color(tag: &Tag) -> &'static str {
    tag.color()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_roundtrips_json() {
        let json = serde_json::to_string(&Tag::ThreeDModel).unwrap();
        assert_eq!(json, "\"3d_model\"");
        let decoded: Tag = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, Tag::ThreeDModel);
    }

    #[test]
    fn tag_fields_match_expected_values() {
        let tag = Tag::ThreeDModel;
        assert_eq!(tag.id(), "3d_model");
        assert_eq!(tag.label(), "3D Model");
        assert!(tag.color().contains("#1d4ed8"));
    }

    #[test]
    fn tag_from_id_resolves_known_tags() {
        assert_eq!(Tag::from_id("vehicle"), Some(Tag::Vehicle));
        assert_eq!(Tag::from_id("unknown"), None);
    }

    #[test]
    fn tag_all_includes_every_variant() {
        let all = Tag::all();
        assert_eq!(all.len(), Tag::iter().count());
        assert!(all.contains(&Tag::Diy));
    }

    #[test]
    fn tag_helpers_expose_fields() {
        let tag = Tag::ThreeDModel;
        assert_eq!(tag_label(&tag), "3D Model");
        assert_eq!(tag_color(&tag), tag.color());
    }
}
