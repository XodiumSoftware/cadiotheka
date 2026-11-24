//! Project wire types shared by the frontend and backend.
//!
//! The backend serializes these structs straight into the D1-backed JSON API,
//! and the frontend deserializes the same definitions, so the contract cannot
//! drift between the two crates.
//!
//! Tag columns deliberately carry wire ids (`Vec<String>`) rather than
//! [`crate::tags::Tag`] values. The tag set evolves independently of stored
//! rows, and a project that references a retired tag must still load; callers
//! resolve ids to [`crate::tags::Tag`] through [`crate::tags::Tag::from_id`]
//! at the point where they need a label or color.

use serde::{Deserialize, Serialize};

use crate::version_state::VersionState;

/// A single IFC file version attached to a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectVersion {
    /// Unique version identifier.
    pub id: String,
    /// Identifier of the project this version belongs to.
    pub project_id: String,
    /// Original filename of the uploaded IFC model.
    pub filename: String,
    /// R2 object key for the stored IFC file.
    pub ifc_key: String,
    /// Maturity state of this version.
    pub state: VersionState,
    /// RFC 3339 timestamp when the version was uploaded.
    pub created_at: String,
    /// Size of the IFC file in bytes.
    #[serde(default)]
    pub file_size: i64,
    /// Semantic version string for this release (e.g. `1.0.0`).
    #[serde(default)]
    pub version: String,
    /// Number of times this version has been downloaded.
    #[serde(default)]
    pub downloads: i64,
}

impl ProjectVersion {
    /// Returns `true` when this version is visible to the public.
    #[must_use]
    pub const fn is_public(&self) -> bool {
        self.state.is_public()
    }
}

/// A Cadiotheka project as exchanged over the API.
///
/// Carries every server-owned field, so the same type is used to serialize a
/// D1 row and to read it back in the browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier.
    pub id: String,
    /// Project title.
    pub title: String,
    /// Author display name, denormalized onto the row for listing.
    pub author: String,
    /// Author account identifier.
    pub author_id: String,
    /// Author username, used for `@author:` filtering and links.
    pub author_username: String,
    /// Account ids of credited collaborators.
    #[serde(with = "crate::json_column")]
    pub collaborator_ids: Vec<String>,
    /// Extended Markdown description shown in the project detail modal.
    pub description: String,
    /// Wire ids of the tags categorizing this content.
    #[serde(with = "crate::json_column")]
    pub tags: Vec<String>,
    /// Download count.
    pub downloads: u64,
    /// Account ids of users who have favorited the project.
    #[serde(with = "crate::json_column")]
    pub favorites: Vec<String>,
    /// RFC 3339 timestamp of the project's last publication or update.
    pub timestamp: String,
    /// IFC file versions attached to the project.
    ///
    /// Populated by the detail endpoint; the listing omits it, so it defaults
    /// to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<ProjectVersion>,
}

impl Project {
    /// Returns `true` when `account_id` is credited as a collaborator.
    #[must_use]
    pub fn has_collaborator(&self, account_id: &str) -> bool {
        self.collaborator_ids.iter().any(|id| id == account_id)
    }

    /// Returns the first public version, which is the one served for viewing.
    #[must_use]
    pub fn latest_public_version(&self) -> Option<&ProjectVersion> {
        self.versions.iter().find(|version| version.is_public())
    }
}

/// Client-editable fields of a project, used for both create and update.
///
/// Server-owned fields (`id`, attribution, `downloads`, `favorites`,
/// `timestamp`, `versions`) are assigned by the backend and ignored when a
/// request body carries them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPayload {
    /// Project title.
    pub title: String,
    /// Account ids of credited collaborators.
    #[serde(default, with = "crate::json_column")]
    pub collaborator_ids: Vec<String>,
    /// Extended Markdown description.
    #[serde(default)]
    pub description: String,
    /// Wire ids of the tags categorizing this content.
    #[serde(default, with = "crate::json_column")]
    pub tags: Vec<String>,
}

/// Partial payload used to patch individual project fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPatch {
    /// New title, when changing it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New tag ids, when replacing them.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_json_column"
    )]
    pub tags: Option<Vec<String>>,
    /// New collaborator ids, when replacing them.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_json_column"
    )]
    pub collaborator_ids: Option<Vec<String>>,
    /// New description, when changing it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Serde adapter for an optional string list stored as JSON text.
///
/// The patch endpoint receives these columns as JSON arrays rather than the
/// JSON-in-TEXT form the stored rows use, so this adapter handles the plain
/// array representation.
mod opt_json_column {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serde's `with` protocol hands the adapter a reference, so the signature
    /// cannot take `Option<&Vec<String>>`.
    #[allow(clippy::ref_option)]
    pub(super) fn serialize<S: Serializer>(
        value: &Option<Vec<String>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value.as_deref() {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<String>>, D::Error> {
        Option::<Vec<String>>::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project() -> Project {
        Project {
            id: "71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2".to_owned(),
            title: "Mountain Bike".to_owned(),
            author: "TrailBlazer".to_owned(),
            author_id: "8af81bd9-b70a-4d64-89e9-83bbc4e0297d".to_owned(),
            author_username: "trailblazer".to_owned(),
            collaborator_ids: vec!["acc-2".to_owned()],
            description: "Extended description.".to_owned(),
            tags: vec!["3d_model".to_owned(), "vehicle".to_owned()],
            downloads: 1200,
            favorites: vec!["acc-3".to_owned()],
            timestamp: "2026-07-07T14:30:00Z".to_owned(),
            versions: Vec::new(),
        }
    }

    #[test]
    fn project_roundtrips_through_json() -> Result<(), serde_json::Error> {
        let project = sample_project();
        let json = serde_json::to_string(&project)?;
        let decoded: Project = serde_json::from_str(&json)?;
        assert_eq!(decoded, project);
        Ok(())
    }

    /// D1 hands back list columns as TEXT holding a JSON array, so the wire
    /// format keeps that shape rather than emitting a nested array.
    #[test]
    fn list_columns_serialize_as_json_text() -> Result<(), serde_json::Error> {
        let json = serde_json::to_value(sample_project())?;
        for field in ["tags", "favorites", "collaborator_ids"] {
            let value = json.get(field).and_then(|v| v.as_str()).unwrap_or_default();
            assert!(value.starts_with('[') && value.ends_with(']'), "{field}");
        }
        Ok(())
    }

    #[test]
    fn omitted_versions_are_not_serialized() -> Result<(), serde_json::Error> {
        let json = serde_json::to_value(sample_project())?;
        assert!(json.get("versions").is_none());
        Ok(())
    }

    #[test]
    fn versions_default_to_empty_when_omitted() -> Result<(), serde_json::Error> {
        let json = r#"{"id":"p1","title":"T","author":"A","author_id":"a1","author_username":"a","collaborator_ids":"[]","description":"E","tags":"[]","downloads":0,"favorites":"[]","timestamp":"2026-07-07T14:30:00Z"}"#;
        let project: Project = serde_json::from_str(json)?;
        assert_eq!(project.versions, Vec::<ProjectVersion>::new());
        Ok(())
    }

    #[test]
    fn collaborator_lookup_matches_by_account_id() {
        let project = sample_project();
        assert!(project.has_collaborator("acc-2"));
        assert!(!project.has_collaborator("acc-9"));
    }

    #[test]
    fn payload_roundtrips_through_json() -> Result<(), serde_json::Error> {
        let payload = ProjectPayload {
            title: "Bike".to_owned(),
            collaborator_ids: vec!["acc-2".to_owned()],
            description: "Model".to_owned(),
            tags: vec!["vehicle".to_owned()],
        };
        let decoded: ProjectPayload = serde_json::from_str(&serde_json::to_string(&payload)?)?;
        assert_eq!(decoded, payload);
        Ok(())
    }

    /// The frontend sends patch bodies with real JSON arrays, not JSON-in-TEXT.
    #[test]
    fn patch_accepts_plain_json_arrays() -> Result<(), serde_json::Error> {
        let patch: ProjectPatch =
            serde_json::from_str(r#"{"tags":["vehicle"],"collaborator_ids":["acc-2"]}"#)?;
        assert_eq!(patch.tags, Some(vec!["vehicle".to_owned()]));
        assert_eq!(patch.collaborator_ids, Some(vec!["acc-2".to_owned()]));
        assert!(patch.title.is_none());
        Ok(())
    }

    #[test]
    fn patch_omits_absent_fields() -> Result<(), serde_json::Error> {
        let patch = ProjectPatch {
            title: Some("New".to_owned()),
            ..ProjectPatch::default()
        };
        let json = serde_json::to_string(&patch)?;
        assert_eq!(json, r#"{"title":"New"}"#);
        Ok(())
    }

    #[test]
    fn latest_public_version_skips_undefined_versions() {
        let mut project = sample_project();
        project.versions = vec![
            ProjectVersion {
                id: "v1".to_owned(),
                project_id: project.id.clone(),
                filename: "a.ifc".to_owned(),
                ifc_key: "ifcs/v1/a.ifc".to_owned(),
                state: VersionState::Undefined,
                created_at: "2026-01-01T00:00:00Z".to_owned(),
                file_size: 1024,
                version: "1.0.0".to_owned(),
                downloads: 0,
            },
            ProjectVersion {
                id: "v2".to_owned(),
                project_id: project.id.clone(),
                filename: "b.ifc".to_owned(),
                ifc_key: "ifcs/v2/b.ifc".to_owned(),
                state: VersionState::Stable,
                created_at: "2026-01-02T00:00:00Z".to_owned(),
                file_size: 2048,
                version: "1.1.0".to_owned(),
                downloads: 42,
            },
        ];
        assert_eq!(
            project.latest_public_version().map(|v| v.id.as_str()),
            Some("v2")
        );
    }
}
