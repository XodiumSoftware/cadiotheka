//! Saved viewer camera and theme state.

/// Serializable camera/theme snapshot used to persist the viewer state.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewState {
    /// World-space camera position, in scene units.
    pub eye: [f32; 3],
    /// World-space point the camera looks at, in scene units.
    pub target: [f32; 3],
    /// World-space up vector that fixes the camera roll; typically `[0, 1, 0]`.
    pub up: [f32; 3],
    /// Background/lighting theme the scene was captured with.
    pub theme: ViewerTheme,
}

impl ViewState {
    /// Serializes the state to a JSON object suitable for storage.
    ///
    /// Returns an empty string if serialization fails, which cannot happen for
    /// this fixed-shape struct.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parses a state previously produced by [`Self::to_json`].
    ///
    /// Returns `None` when the input is not valid JSON or does not match the
    /// expected shape, for example after a storage schema change.
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// Viewer background/lighting theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerTheme {
    /// Dark background and lighting, the default viewer look.
    #[default]
    Dark,
    /// Light background and lighting, suited to printing-style previews.
    Light,
}

/// Direction from which the camera should look at the scene center.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewDirection {
    /// Looking along `-Z` toward the scene center.
    Front,
    /// Looking along `+Z` toward the scene center.
    Back,
    /// Looking along `+X` toward the scene center.
    Left,
    /// Looking along `-X` toward the scene center.
    Right,
    /// Looking down the `-Y` axis from above the scene.
    Top,
    /// Looking up the `+Y` axis from below the scene.
    Bottom,
    /// Diagonal between [`Self::Front`] and [`Self::Right`].
    FrontRight,
    /// Diagonal between [`Self::Back`] and [`Self::Right`].
    BackRight,
    /// Diagonal between [`Self::Back`] and [`Self::Left`].
    BackLeft,
    /// Diagonal between [`Self::Front`] and [`Self::Left`].
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
