//! Content tags and categories for Cadiotheka.

use strum::{Display, EnumIter, IntoStaticStr};

/// Predefined content tags used to categorize cards and enable filtering.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    EnumIter,
    Display,
    IntoStaticStr,
)]
pub enum Tag {
    /// 3D models and meshes.
    #[serde(rename = "3d_model")]
    #[strum(serialize = "3D Model")]
    Model3d,
    /// 2D drawings, blueprints, or schematics.
    #[serde(rename = "2d_drawing")]
    #[strum(serialize = "2D Drawing")]
    Drawing2d,
    /// Parametric or script-driven designs.
    #[serde(rename = "parametric")]
    #[strum(serialize = "Parametric")]
    Parametric,
    /// Physical parts intended for fabrication.
    #[serde(rename = "fabrication")]
    #[strum(serialize = "Fabrication")]
    Fabrication,
    /// Robotics, mechanisms, or moving assemblies.
    #[serde(rename = "robotics")]
    #[strum(serialize = "Robotics")]
    Robotics,
    /// Furniture and interior objects.
    #[serde(rename = "furniture")]
    #[strum(serialize = "Furniture")]
    Furniture,
    /// Vehicles and transportation.
    #[serde(rename = "vehicle")]
    #[strum(serialize = "Vehicle")]
    Vehicle,
    /// Architectural structures and spaces.
    #[serde(rename = "architecture")]
    #[strum(serialize = "Architecture")]
    Architecture,
    /// Electronics, PCBs, and wiring.
    #[serde(rename = "electronics")]
    #[strum(serialize = "Electronics")]
    Electronics,
    /// Tools, jigs, and workshop accessories.
    #[serde(rename = "tooling")]
    #[strum(serialize = "Tooling")]
    Tooling,
    /// Lighting and light fixtures.
    #[serde(rename = "lighting")]
    #[strum(serialize = "Lighting")]
    Lighting,
    /// Do-it-yourself projects and hobby builds.
    #[serde(rename = "diy")]
    #[strum(serialize = "DIY")]
    Diy,
    /// Interior design and household objects.
    #[serde(rename = "interior")]
    #[strum(serialize = "Interior")]
    Interior,
    /// Mechanical or structural engineering.
    #[serde(rename = "engineering")]
    #[strum(serialize = "Engineering")]
    Engineering,
    /// Aerospace and aviation.
    #[serde(rename = "aerospace")]
    #[strum(serialize = "Aerospace")]
    Aerospace,
    /// Decorative objects and ornaments.
    #[serde(rename = "decor")]
    #[strum(serialize = "Decor")]
    Decor,
    /// Medical devices, prosthetics, and anatomy.
    #[serde(rename = "medical")]
    #[strum(serialize = "Medical")]
    Medical,
    /// Game-ready assets.
    #[serde(rename = "game_asset")]
    #[strum(serialize = "Game Asset")]
    GameAsset,
    /// Art, sculptures, and decorative objects.
    #[serde(rename = "art")]
    #[strum(serialize = "Art")]
    Art,
    /// Educational or tutorial content.
    #[serde(rename = "educational")]
    #[strum(serialize = "Educational")]
    Educational,
    /// Work in progress or experimental content.
    #[serde(rename = "wip")]
    #[strum(serialize = "WIP")]
    WorkInProgress,
}

impl Tag {
    /// Returns the user-facing label for this tag.
    pub fn label(&self) -> &'static str {
        self.into()
    }

    /// Returns a Tailwind-compatible CSS color class for this tag.
    pub const fn color(&self) -> &'static str {
        match self {
            Self::Model3d => "bg-blue-700 text-white",
            Self::Drawing2d => "bg-cyan-700 text-white",
            Self::Parametric => "bg-purple-700 text-white",
            Self::Fabrication => "bg-orange-700 text-white",
            Self::Robotics => "bg-red-700 text-white",
            Self::Furniture => "bg-amber-800 text-white",
            Self::Vehicle => "bg-green-700 text-white",
            Self::Architecture => "bg-gray-700 text-white",
            Self::Electronics => "bg-yellow-600 text-white",
            Self::Tooling => "bg-slate-600 text-white",
            Self::Lighting => "bg-amber-600 text-white",
            Self::Diy => "bg-orange-600 text-white",
            Self::Interior => "bg-rose-700 text-white",
            Self::Engineering => "bg-slate-700 text-white",
            Self::Aerospace => "bg-sky-700 text-white",
            Self::Decor => "bg-rose-600 text-white",
            Self::Medical => "bg-emerald-700 text-white",
            Self::GameAsset => "bg-pink-700 text-white",
            Self::Art => "bg-fuchsia-700 text-white",
            Self::Educational => "bg-teal-700 text-white",
            Self::WorkInProgress => "bg-lime-700 text-white",
        }
    }
}

/// Convenience accessor for a tag's user-facing label.
pub fn tag_label(tag: &Tag) -> &'static str {
    tag.label()
}

/// Convenience accessor for a tag's Tailwind color class.
pub fn tag_color(tag: &Tag) -> &'static str {
    tag.color()
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn tag_label_roundtrips() {
        assert_eq!(Tag::Model3d.label(), "3D Model");
        assert_eq!(Tag::WorkInProgress.label(), "WIP");
    }

    #[test]
    fn tag_serialization_uses_rename() {
        let json = serde_json::to_string(&Tag::Model3d).unwrap();
        assert_eq!(json, "\"3d_model\"");
        let tag: Tag = serde_json::from_str("\"3d_model\"").unwrap();
        assert_eq!(tag, Tag::Model3d);
    }

    #[test]
    fn tag_iteration_covers_all_variants() {
        assert_eq!(Tag::iter().count(), 21);
    }
}
