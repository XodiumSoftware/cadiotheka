//! IFC file version maturity states.

use serde::{Deserialize, Serialize};

/// The maturity state of a project IFC file version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionState {
    /// Newly uploaded, only visible to project editors.
    Undefined,
    /// Early/experimental version.
    Alpha,
    /// Work-in-progress but usable version.
    Beta,
    /// Production-ready version.
    Stable,
}

impl VersionState {
    /// All states in the order they appear in the dropdown.
    pub const VARIANTS: &[Self] = &[Self::Undefined, Self::Alpha, Self::Beta, Self::Stable];

    /// Returns a text color utility class representing the state's severity.
    pub const fn color_class(self) -> &'static str {
        match self {
            Self::Undefined => "text-base-content/50",
            Self::Alpha => "text-error",
            Self::Beta => "text-warning",
            Self::Stable => "text-success",
        }
    }

    /// Returns the `DaisyUI` badge class used to color the version icon.
    pub const fn badge_class(self) -> &'static str {
        match self {
            Self::Undefined => "badge-ghost",
            Self::Alpha => "badge-error",
            Self::Beta => "badge-warning",
            Self::Stable => "badge-success",
        }
    }

    /// Returns the human-readable label for the state.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Undefined => "Undefined",
            Self::Alpha => "Alpha",
            Self::Beta => "Beta",
            Self::Stable => "Release",
        }
    }

    /// Returns the single-letter abbreviation shown in the versions table.
    pub const fn letter(self) -> &'static str {
        match self {
            Self::Undefined => "-",
            Self::Alpha => "A",
            Self::Beta => "B",
            Self::Stable => "R",
        }
    }

    /// Returns true if the version is visible to the public.
    pub const fn is_public(self) -> bool {
        !matches!(self, Self::Undefined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_list_contains_all_states() {
        assert_eq!(VersionState::VARIANTS.len(), 4);
    }

    #[test]
    fn version_state_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&VersionState::Undefined).unwrap(),
            "\"undefined\""
        );
        assert_eq!(
            serde_json::to_string(&VersionState::Alpha).unwrap(),
            "\"alpha\""
        );
        assert_eq!(
            serde_json::to_string(&VersionState::Beta).unwrap(),
            "\"beta\""
        );
        assert_eq!(
            serde_json::to_string(&VersionState::Stable).unwrap(),
            "\"stable\""
        );
    }

    #[test]
    fn undefined_is_not_public() {
        assert!(!VersionState::Undefined.is_public());
        assert!(VersionState::Alpha.is_public());
        assert!(VersionState::Beta.is_public());
        assert!(VersionState::Stable.is_public());
    }
}
