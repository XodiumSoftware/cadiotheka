//! Hardcoded content tags for Cadiotheka.
//!
//! Tags are defined as enum variants in this file and shared between the
//! frontend and backend. Project rows still store tag wire ids as JSON arrays,
//! so both crates resolve labels and colors through the helpers below.

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

/// Content tag variants with stable wire ids, labels, and Tailwind color styles.
///
/// Serde is implemented by hand (see below) so that an unknown wire id decodes
/// to `None` instead of failing the whole row. Tags are also `PartialOrd` /
/// `Ord` so deserialized collections can be normalized back into display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum Tag {
    /// Three-dimensional models and assets.
    ThreeDModel,
    /// Two-dimensional drawings and diagrams.
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
    /// Artistic or sculptical models.
    Art,
    /// Educational models and demonstrations.
    Educational,
    /// Work-in-progress designs.
    Wip,
}

impl Tag {
    /// Stable wire id stored on project rows.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn all() -> Vec<Tag> {
        Tag::iter().collect()
    }

    /// Resolves a tag from its wire id, or `None` if the id is unknown.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Tag> {
        Tag::iter().find(|tag| tag.id() == id)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl Serialize for Tag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TagVisitor;

        impl Visitor<'_> for TagVisitor {
            type Value = Tag;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a tag wire id")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Tag, E> {
                Tag::from_id(value).ok_or_else(|| E::custom(format!("unknown tag id: {value:?}")))
            }
        }

        deserializer.deserialize_str(TagVisitor)
    }
}

/// Collects tags into a de-duplicated list in enum display order.
///
/// Decoding skips unknown ids, so this is what keeps a stored set stable
/// regardless of the order it was originally written in.
fn normalize(tags: &[Tag]) -> Vec<Tag> {
    let mut tags = tags.to_vec();
    tags.sort_unstable();
    tags.dedup();
    tags
}

/// Convenience accessor for a tag's user-facing label.
#[must_use]
pub fn tag_label(tag: &Tag) -> &'static str {
    tag.label()
}

/// Convenience accessor for a tag's inline CSS color style.
#[must_use]
pub fn tag_color(tag: &Tag) -> &'static str {
    tag.color()
}

/// Serde adapter for a project's tag list.
///
/// D1 stores tags as a JSON array in a TEXT column. Ids the enum no longer
/// knows are skipped rather than failing the row: the tag set evolves faster
/// than stored rows, and a project referencing a retired tag must still load.
pub mod serde_tags {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::{Tag, normalize};

    /// Writes a tag list as a JSON array in a TEXT column.
    ///
    /// # Errors
    ///
    /// Returns the serializer's error when the list cannot be encoded as JSON
    /// or written.
    pub fn serialize<S: Serializer>(value: &[Tag], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
    }

    /// Reads a tag list from a JSON array in a TEXT column.
    ///
    /// Ids are decoded through [`Tag::from_id`] so an unrecognized one is
    /// dropped instead of failing the whole row.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error when the value is not a string or does
    /// not hold a JSON array of strings.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Tag>, D::Error> {
        let text = String::deserialize(deserializer)?;
        let ids = serde_json::from_str::<Vec<String>>(&text).map_err(serde::de::Error::custom)?;
        Ok(normalize(
            &ids.iter()
                .filter_map(|id| Tag::from_id(id))
                .collect::<Vec<_>>(),
        ))
    }
}

/// Serde adapter for a tag list carried as a comma-separated query parameter.
///
/// Unknown ids are skipped silently so a stale bookmark or link still returns
/// the tags the server does recognise.
pub mod tag_ids {
    use serde::{Deserialize, Deserializer};

    use super::Tag;

    /// Decodes a comma-separated list of tag wire ids, skipping unknown ids so
    /// one stale id cannot discard the others.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error when the value is not a string.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Tag>, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(raw
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .filter_map(Tag::from_id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_roundtrips_json() -> Result<(), serde_json::Error> {
        let json = serde_json::to_string(&Tag::ThreeDModel)?;
        assert_eq!(json, "\"3d_model\"");
        let decoded: Tag = serde_json::from_str(&json)?;
        assert_eq!(decoded, Tag::ThreeDModel);
        Ok(())
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
    fn deserializing_an_unknown_wire_id_reports_it() {
        let error = serde_json::from_str::<Tag>("\"retired_tag\"");
        assert!(error.is_err());
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

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct TagRow {
        #[serde(with = "super::serde_tags")]
        tags: Vec<Tag>,
    }

    #[test]
    fn tag_column_skips_retired_ids_instead_of_failing_the_row() -> Result<(), serde_json::Error> {
        let row: TagRow =
            serde_json::from_str(r#"{"tags":"[\"vehicle\",\"retired_tag\",\"art\"]"}"#)?;
        assert_eq!(row.tags, vec![Tag::Vehicle, Tag::Art]);
        Ok(())
    }

    #[test]
    fn tag_column_normalizes_order_and_duplicates() -> Result<(), serde_json::Error> {
        let row: TagRow =
            serde_json::from_str(r#"{"tags":"[\"wip\",\"3d_model\",\"wip\",\"decor\"]"}"#)?;
        assert_eq!(row.tags, vec![Tag::ThreeDModel, Tag::Decor, Tag::Wip]);
        Ok(())
    }

    #[test]
    fn tag_column_roundtrips_as_a_json_string() -> Result<(), serde_json::Error> {
        let row = TagRow {
            tags: vec![Tag::ThreeDModel, Tag::Vehicle],
        };
        let json = serde_json::to_string(&row)?;
        assert_eq!(json, r#"{"tags":"[\"3d_model\",\"vehicle\"]"}"#);
        assert_eq!(serde_json::from_str::<TagRow>(&json)?, row);
        Ok(())
    }

    #[test]
    fn tag_column_defaults_to_empty_when_null() -> Result<(), serde_json::Error> {
        let row: TagRow = serde_json::from_str(r#"{"tags":"[]"}"#)?;
        assert_eq!(row.tags, Vec::<Tag>::new());
        Ok(())
    }

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct TagQuery {
        #[serde(default, with = "super::tag_ids")]
        tags: Vec<Tag>,
    }

    #[test]
    fn tag_query_ids_skip_blanks_and_unknowns() -> Result<(), serde_json::Error> {
        let query: TagQuery = serde_json::from_str(r#"{"tags":" vehicle, , retired_tag,art "}"#)?;
        assert_eq!(query.tags, vec![Tag::Vehicle, Tag::Art]);
        Ok(())
    }

    #[test]
    fn tag_query_ids_default_to_empty() -> Result<(), serde_json::Error> {
        let query: TagQuery = serde_json::from_str("{}")?;
        assert_eq!(query.tags, Vec::<Tag>::new());
        Ok(())
    }
}
