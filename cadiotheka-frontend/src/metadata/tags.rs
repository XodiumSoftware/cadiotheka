//! Content tags and categories for Cadiotheka.

/// Predefined content tags used to categorize cards and enable filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Tag {
    /// 3D models and meshes.
    #[serde(rename = "3d_model")]
    Model3d,
    /// 2D drawings, blueprints, or schematics.
    #[serde(rename = "2d_drawing")]
    Drawing2d,
    /// Parametric or script-driven designs.
    #[serde(rename = "parametric")]
    Parametric,
    /// Physical parts intended for fabrication.
    #[serde(rename = "fabrication")]
    Fabrication,
    /// Robotics, mechanisms, or moving assemblies.
    #[serde(rename = "robotics")]
    Robotics,
    /// Furniture and interior objects.
    #[serde(rename = "furniture")]
    Furniture,
    /// Vehicles and transportation.
    #[serde(rename = "vehicle")]
    Vehicle,
    /// Architectural structures and spaces.
    #[serde(rename = "architecture")]
    Architecture,
    /// Electronics, PCBs, and wiring.
    #[serde(rename = "electronics")]
    Electronics,
    /// Tools, jigs, and workshop accessories.
    #[serde(rename = "tooling")]
    Tooling,
    /// Lighting and light fixtures.
    #[serde(rename = "lighting")]
    Lighting,
    /// Do-it-yourself projects and hobby builds.
    #[serde(rename = "diy")]
    Diy,
    /// Interior design and household objects.
    #[serde(rename = "interior")]
    Interior,
    /// Mechanical or structural engineering.
    #[serde(rename = "engineering")]
    Engineering,
    /// Aerospace and aviation.
    #[serde(rename = "aerospace")]
    Aerospace,
    /// Decorative objects and ornaments.
    #[serde(rename = "decor")]
    Decor,
    /// Medical devices, prosthetics, and anatomy.
    #[serde(rename = "medical")]
    Medical,
    /// Game-ready assets.
    #[serde(rename = "game_asset")]
    GameAsset,
    /// Art, sculptures, and decorative objects.
    #[serde(rename = "art")]
    Art,
    /// Educational or tutorial content.
    #[serde(rename = "educational")]
    Educational,
    /// Work in progress or experimental content.
    #[serde(rename = "wip")]
    WorkInProgress,
}

impl Tag {
    /// Returns the user-facing label for this tag.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Model3d => "3D Model",
            Self::Drawing2d => "2D Drawing",
            Self::Parametric => "Parametric",
            Self::Fabrication => "Fabrication",
            Self::Robotics => "Robotics",
            Self::Furniture => "Furniture",
            Self::Vehicle => "Vehicle",
            Self::Architecture => "Architecture",
            Self::Electronics => "Electronics",
            Self::Tooling => "Tooling",
            Self::Lighting => "Lighting",
            Self::Diy => "DIY",
            Self::Interior => "Interior",
            Self::Engineering => "Engineering",
            Self::Aerospace => "Aerospace",
            Self::Decor => "Decor",
            Self::Medical => "Medical",
            Self::GameAsset => "Game Asset",
            Self::Art => "Art",
            Self::Educational => "Educational",
            Self::WorkInProgress => "WIP",
        }
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

    /// All available tags in a stable order.
    pub const fn all() -> [Self; 21] {
        [
            Self::Model3d,
            Self::Drawing2d,
            Self::Parametric,
            Self::Fabrication,
            Self::Robotics,
            Self::Furniture,
            Self::Vehicle,
            Self::Architecture,
            Self::Electronics,
            Self::Tooling,
            Self::Lighting,
            Self::Diy,
            Self::Interior,
            Self::Engineering,
            Self::Aerospace,
            Self::Decor,
            Self::Medical,
            Self::GameAsset,
            Self::Art,
            Self::Educational,
            Self::WorkInProgress,
        ]
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
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
}
