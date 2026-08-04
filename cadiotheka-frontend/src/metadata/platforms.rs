//! Supported CAD platforms for Cadiotheka content.

use strum::{Display, EnumIter, IntoStaticStr};

/// Predefined platforms that a card may support.
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
pub enum Platform {
    /// Blender (<https://www.blender.org>).
    #[serde(rename = "blender")]
    #[strum(serialize = "Blender")]
    Blender,
    /// `FreeCAD` (<https://www.freecad.org>).
    #[serde(rename = "freecad")]
    #[strum(serialize = "FreeCAD")]
    FreeCAD,
    /// `SketchUp` (<https://www.sketchup.com>).
    #[serde(rename = "sketchup")]
    #[strum(serialize = "SketchUp")]
    SketchUp,
    /// Autodesk Fusion 360.
    #[serde(rename = "fusion_360")]
    #[strum(serialize = "Fusion 360")]
    Fusion360,
    /// `KiCad` (<https://www.kicad.org>).
    #[serde(rename = "kicad")]
    #[strum(serialize = "KiCad")]
    KiCad,
    /// `AutoCAD`.
    #[serde(rename = "autocad")]
    #[strum(serialize = "AutoCAD")]
    AutoCAD,
    /// `SolidWorks`.
    #[serde(rename = "solidworks")]
    #[strum(serialize = "SolidWorks")]
    SolidWorks,
    /// Onshape.
    #[serde(rename = "onshape")]
    #[strum(serialize = "Onshape")]
    Onshape,
    /// Tinkercad.
    #[serde(rename = "tinkercad")]
    #[strum(serialize = "Tinkercad")]
    Tinkercad,
    /// Generic STEP/IGES-compatible CAD.
    #[serde(rename = "step")]
    #[strum(serialize = "STEP")]
    Step,
    /// Generic STL/OBJ mesh tools.
    #[serde(rename = "mesh")]
    #[strum(serialize = "Mesh")]
    Mesh,
}

impl Platform {
    /// Returns the user-facing label for this platform.
    pub fn label(&self) -> &'static str {
        self.into()
    }

    /// Returns a Tailwind-compatible CSS color class for this platform.
    pub const fn color(self) -> &'static str {
        match self {
            Self::Blender => "text-orange-700",
            Self::FreeCAD => "text-blue-700",
            Self::SketchUp => "text-red-700",
            Self::Fusion360 => "text-yellow-700",
            Self::KiCad => "text-green-700",
            Self::AutoCAD => "text-red-900",
            Self::SolidWorks => "text-red-800",
            Self::Onshape => "text-gray-700",
            Self::Tinkercad => "text-cyan-700",
            Self::Step | Self::Mesh => "text-gray-600",
        }
    }
}

/// Convenience accessor for a platform's user-facing label.
pub fn platform_label(platform: &Platform) -> &'static str {
    platform.label()
}

/// Convenience accessor for a platform's Tailwind color class.
pub fn platform_color(platform: &Platform) -> &'static str {
    platform.color()
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn platform_label_roundtrips() {
        assert_eq!(Platform::Blender.label(), "Blender");
        assert_eq!(Platform::Fusion360.label(), "Fusion 360");
    }

    #[test]
    fn platform_serialization_uses_rename() {
        let json = serde_json::to_string(&Platform::FreeCAD).unwrap();
        assert_eq!(json, "\"freecad\"");
        let platform: Platform = serde_json::from_str("\"freecad\"").unwrap();
        assert_eq!(platform, Platform::FreeCAD);
    }

    #[test]
    fn platform_iteration_covers_all_variants() {
        assert_eq!(Platform::iter().count(), 11);
    }
}
