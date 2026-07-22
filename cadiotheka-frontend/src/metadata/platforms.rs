//! Supported CAD platforms for Cadiotheka content.

/// Predefined platforms that a card may support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Platform {
    /// Blender (https://www.blender.org).
    #[serde(rename = "blender")]
    Blender,
    /// FreeCAD (https://www.freecad.org).
    #[serde(rename = "freecad")]
    FreeCAD,
    /// SketchUp (https://www.sketchup.com).
    #[serde(rename = "sketchup")]
    SketchUp,
    /// Autodesk Fusion 360.
    #[serde(rename = "fusion_360")]
    Fusion360,
    /// KiCad (https://www.kicad.org).
    #[serde(rename = "kicad")]
    KiCad,
    /// AutoCAD.
    #[serde(rename = "autocad")]
    AutoCAD,
    /// SolidWorks.
    #[serde(rename = "solidworks")]
    SolidWorks,
    /// Onshape.
    #[serde(rename = "onshape")]
    Onshape,
    /// Tinkercad.
    #[serde(rename = "tinkercad")]
    Tinkercad,
    /// Generic STEP/IGES-compatible CAD.
    #[serde(rename = "step")]
    Step,
    /// Generic STL/OBJ mesh tools.
    #[serde(rename = "mesh")]
    Mesh,
}

impl Platform {
    /// Returns the user-facing label for this platform.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Blender => "Blender",
            Self::FreeCAD => "FreeCAD",
            Self::SketchUp => "SketchUp",
            Self::Fusion360 => "Fusion 360",
            Self::KiCad => "KiCad",
            Self::AutoCAD => "AutoCAD",
            Self::SolidWorks => "SolidWorks",
            Self::Onshape => "Onshape",
            Self::Tinkercad => "Tinkercad",
            Self::Step => "STEP",
            Self::Mesh => "Mesh",
        }
    }

    /// Returns a Tailwind-compatible CSS color class for this platform.
    pub const fn color(&self) -> &'static str {
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
            Self::Step => "text-gray-600",
            Self::Mesh => "text-gray-600",
        }
    }

    /// All available platforms in a stable order.
    pub const fn all() -> [Self; 11] {
        [
            Self::Blender,
            Self::FreeCAD,
            Self::SketchUp,
            Self::Fusion360,
            Self::KiCad,
            Self::AutoCAD,
            Self::SolidWorks,
            Self::Onshape,
            Self::Tinkercad,
            Self::Step,
            Self::Mesh,
        ]
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
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
}
