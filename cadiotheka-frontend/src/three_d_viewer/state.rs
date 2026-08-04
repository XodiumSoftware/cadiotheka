//! Saved viewer camera and theme state.

/// Serializable camera/theme snapshot used to persist the viewer state.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewState {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub theme: ViewerTheme,
    pub shadows: bool,
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
    pub show_debug: bool,
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
            show_debug: false,
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
