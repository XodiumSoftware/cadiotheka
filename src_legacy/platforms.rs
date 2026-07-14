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

    /// Returns the display color for this platform.
    pub const fn color(&self) -> egui::Color32 {
        match self {
            Self::Blender => egui::Color32::from_rgb(232, 119, 37),
            Self::FreeCAD => egui::Color32::from_rgb(59, 130, 246),
            Self::SketchUp => egui::Color32::from_rgb(239, 68, 68),
            Self::Fusion360 => egui::Color32::from_rgb(234, 179, 8),
            Self::KiCad => egui::Color32::from_rgb(34, 197, 94),
            Self::AutoCAD => egui::Color32::from_rgb(153, 27, 27),
            Self::SolidWorks => egui::Color32::from_rgb(220, 38, 38),
            Self::Onshape => egui::Color32::from_rgb(108, 117, 125),
            Self::Tinkercad => egui::Color32::from_rgb(6, 182, 212),
            Self::Step => egui::Color32::from_rgb(128, 128, 128),
            Self::Mesh => egui::Color32::from_rgb(192, 192, 192),
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
