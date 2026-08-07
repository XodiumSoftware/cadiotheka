//! Saved viewer camera and theme state.

/// Serializable camera/theme snapshot used to persist the viewer state.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewState {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub theme: ViewerTheme,
}

impl ViewState {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// Serializable per-project viewer display toggles.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewerSettings {
    pub show_grid: bool,
    pub show_axes: bool,
}

impl ViewerSettings {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

impl Default for ViewerSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_axes: true,
        }
    }
}

/// Viewer background/lighting theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViewerTheme {
    #[default]
    Dark,
    Light,
}

/// Direction from which the camera should look at the scene center.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewDirection {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
    FrontRight,
    BackRight,
    BackLeft,
    FrontLeft,
}

impl ViewDirection {
    /// Human-readable label shown in the gizmo tooltip.
    pub fn label(self) -> &'static str {
        match self {
            Self::Front => "Front",
            Self::Back => "Back",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::FrontRight => "Front-Right",
            Self::BackRight => "Back-Right",
            Self::BackLeft => "Back-Left",
            Self::FrontLeft => "Front-Left",
        }
    }

    /// Computes the world-space eye offset and up vector for this direction.
    ///
    /// `distance` is the distance from the scene center to the camera eye.
    pub fn eye_and_up(self, distance: f32) -> (three_d_asset::Vec3, three_d_asset::Vec3) {
        use three_d_asset::vec3;
        let sqrt2_inv = 1.0 / 2.0_f32.sqrt();
        match self {
            Self::Front => (vec3(0.0, 0.0, distance), vec3(0.0, 1.0, 0.0)),
            Self::Back => (vec3(0.0, 0.0, -distance), vec3(0.0, 1.0, 0.0)),
            Self::Left => (vec3(-distance, 0.0, 0.0), vec3(0.0, 1.0, 0.0)),
            Self::Right => (vec3(distance, 0.0, 0.0), vec3(0.0, 1.0, 0.0)),
            Self::Top => (vec3(0.0, distance, 0.0), vec3(0.0, 0.0, -1.0)),
            Self::Bottom => (vec3(0.0, -distance, 0.0), vec3(0.0, 0.0, 1.0)),
            Self::FrontRight => (
                vec3(distance * sqrt2_inv, 0.0, distance * sqrt2_inv),
                vec3(0.0, 1.0, 0.0),
            ),
            Self::BackRight => (
                vec3(distance * sqrt2_inv, 0.0, -distance * sqrt2_inv),
                vec3(0.0, 1.0, 0.0),
            ),
            Self::BackLeft => (
                vec3(-distance * sqrt2_inv, 0.0, -distance * sqrt2_inv),
                vec3(0.0, 1.0, 0.0),
            ),
            Self::FrontLeft => (
                vec3(-distance * sqrt2_inv, 0.0, distance * sqrt2_inv),
                vec3(0.0, 1.0, 0.0),
            ),
        }
    }
}
