//! Project data models shared with the backend.
//!
//! [`Project`] and [`ProjectVersion`] are the same types the backend
//! serializes, so the wire contract cannot drift between the two crates. This
//! module adds the browser-only helpers on top: download URLs and the local
//! clock.

use crate::utils::ifc_url;
use serde::{Deserialize, Serialize};
pub use shared::projects::{Project, ProjectVersion};
pub use shared::version_state::VersionState;

/// A single IFC file version returned by the backend after an upload.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VersionUploadResponse {
    /// R2 object key for the stored IFC file.
    pub ifc_key: String,
    /// Unique version identifier.
    pub version_id: String,
    /// Size of the IFC file in bytes.
    pub file_size: i64,
    /// Semantic version string for this release.
    pub version: String,
    /// Number of times this version has been downloaded.
    pub downloads: i64,
}

/// Builds the frontend download URL for an IFC version.
///
/// Uses the version's stored R2 object key (`ifcs/{id}/{filename}`) so the
/// browser can fetch the file from the public asset domain directly. Deriving
/// the key from `id` and `filename` instead would break legacy versions whose
/// key does not follow that pattern.
pub fn ifc_download_url(version: &ProjectVersion) -> String {
    ifc_url(&version.ifc_key)
}

/// Returns the public download URL for the latest visible IFC version, if any.
pub fn latest_visible_ifc_url(versions: &[ProjectVersion]) -> Option<String> {
    versions
        .iter()
        .find(|version| version.state.is_public())
        .map(ifc_download_url)
}

/// Returns the current UTC time using the JavaScript `Date` API.
pub fn now_utc() -> time::OffsetDateTime {
    shared::time::now_utc()
}

/// Creates a new project payload for submission to the backend.
///
/// The backend assigns the `id` and `timestamp`, attributes the project to the
/// authenticated account, and initializes the `downloads`/`favorites`
/// counters. The values generated here for those fields are placeholders that
/// the server overrides.
pub fn new_project_payload(title: String, description: String, tags: Vec<String>) -> Project {
    Project {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        author: String::new(),
        author_id: String::new(),
        author_username: String::new(),
        collaborator_ids: vec![],
        description,
        tags,
        downloads: 0,
        favorites: vec![],
        timestamp: now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        versions: vec![],
    }
}

/// Outcome of attempting to create a project on the backend.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ProjectCreationResult {
    /// The project was created successfully.
    Created(Project),
    /// The backend rejected one or more fields; map keys are field names.
    ValidationErrors(std::collections::HashMap<String, String>),
    /// A network, serialization, or unexpected server failure occurred.
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn sample_project() -> Project {
        Project {
            id: "71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2".to_owned(),
            title: "Mountain Bike".to_owned(),
            author: "TrailBlazer".to_owned(),
            author_id: "8af81bd9-b70a-4d64-89e9-83bbc4e0297d".to_owned(),
            author_username: "trailblazer".to_owned(),
            collaborator_ids: vec![],
            description: "Extended description.".to_owned(),
            tags: vec!["3d_model".to_owned(), "vehicle".to_owned()],
            downloads: 1200,
            favorites: vec![
                "11111111-1111-1111-1111-111111111111".to_owned(),
                "22222222-2222-2222-2222-222222222222".to_owned(),
            ],
            timestamp: "2026-07-07T14:30:00Z".to_owned(),
            versions: vec![],
        }
    }

    fn sample_version(id: &str, ifc_key: &str, state: VersionState) -> ProjectVersion {
        ProjectVersion {
            id: id.to_owned(),
            project_id: "p1".to_owned(),
            filename: "model.ifc".to_owned(),
            ifc_key: ifc_key.to_owned(),
            state,
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            file_size: 1024,
            version: "1.0.0".to_owned(),
            downloads: 0,
        }
    }

    #[test]
    fn latest_visible_ifc_url_prefers_first_public_version() {
        let versions = vec![
            sample_version("v1", "ifcs/v1/a.ifc", VersionState::Undefined),
            sample_version("v2", "ifcs/v2/b.ifc", VersionState::Stable),
        ];
        assert_eq!(
            latest_visible_ifc_url(&versions),
            Some(ifc_url("ifcs/v2/b.ifc"))
        );
    }

    #[test]
    fn latest_visible_ifc_url_returns_none_when_all_undefined() {
        let versions = vec![sample_version(
            "v1",
            "ifcs/v1/a.ifc",
            VersionState::Undefined,
        )];
        assert_eq!(latest_visible_ifc_url(&versions), None);
    }

    #[test]
    fn project_deserializes_backend_json_string_columns() -> Result<(), serde_json::Error> {
        let json = r#"[{"id":"71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2","title":"Mountain Bike","author":"TrailBlazer","author_id":"8af81bd9-b70a-4d64-89e9-83bbc4e0297d","author_username":"trailblazer","collaborator_ids":"[]","description":"Extended.","tags":"[\"3d_model\",\"vehicle\",\"fabrication\",\"engineering\",\"diy\"]","downloads":1200,"favorites":"[\"11111111-1111-1111-1111-111111111111\",\"22222222-2222-2222-2222-222222222222\"]","timestamp":"2026-07-07T14:30:00Z"}]"#;
        let projects: Vec<Project> = serde_json::from_str(json)?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].title, "Mountain Bike");
        assert_eq!(projects[0].tags.len(), 5);
        assert_eq!(projects[0].favorites.len(), 2);
        Ok(())
    }

    #[test]
    fn project_serializes_and_deserializes() -> Result<(), serde_json::Error> {
        let project = sample_project();
        let json = serde_json::to_string(&project)?;
        let decoded: Project = serde_json::from_str(&json)?;
        assert_eq!(decoded, project);
        Ok(())
    }

    #[test]
    fn project_serializes_json_string_columns_for_backend() -> Result<(), serde_json::Error> {
        let project = sample_project();
        let value = serde_json::to_value(&project)?;
        for field in ["tags", "favorites", "collaborator_ids"] {
            let column = value
                .get(field)
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            assert!(column.starts_with('[') && column.ends_with(']'), "{field}");
        }
        Ok(())
    }

    #[test]
    fn project_deserializes_empty_json_string_columns() -> Result<(), serde_json::Error> {
        let json = r#"[{"id":"p1","title":"T","author":"A","author_id":"a1","author_username":"a","collaborator_ids":"[]","description":"E","tags":"[]","downloads":0,"favorites":"[]","timestamp":"2026-07-07T14:30:00Z"}]"#;
        let projects: Vec<Project> = serde_json::from_str(json)?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].tags, Vec::<String>::new());
        assert_eq!(projects[0].favorites, Vec::<String>::new());
        assert_eq!(projects[0].collaborator_ids, Vec::<String>::new());
        Ok(())
    }

    /// The URL points at the stored R2 object key, so legacy versions whose key
    /// does not match `ifcs/{id}/{filename}` still resolve correctly.
    #[test]
    fn ifc_download_url_uses_stored_object_key() {
        let version = sample_version(
            "vid-123",
            "ifcs/legacy-project-id/model.ifc",
            VersionState::Stable,
        );
        assert_eq!(
            ifc_download_url(&version),
            ifc_url("ifcs/legacy-project-id/model.ifc")
        );
    }

    /// A project referencing a tag that no longer exists must still load.
    #[test]
    fn project_tolerates_unknown_tag_ids() -> Result<(), serde_json::Error> {
        let json = r#"{"id":"p1","title":"T","author":"A","author_id":"a1","author_username":"a","collaborator_ids":"[]","description":"E","tags":"[\"vehicle\",\"retired_tag\"]","downloads":0,"favorites":"[]","timestamp":"2026-07-07T14:30:00Z"}"#;
        let project: Project = serde_json::from_str(json)?;
        assert_eq!(project.tags, vec!["vehicle", "retired_tag"]);
        Ok(())
    }

    #[test]
    fn project_payload_omits_server_owned_fields() -> Result<(), serde_json::Error> {
        let payload = shared::projects::ProjectPayload {
            title: "Bike".to_owned(),
            collaborator_ids: vec![],
            description: "Model".to_owned(),
            tags: vec!["vehicle".to_owned()],
        };
        let wire: serde_json::Value = serde_json::to_value(&payload)?;
        for server_owned in ["id", "author", "author_id", "downloads", "favorites"] {
            assert!(wire.get(server_owned).is_none(), "{server_owned}");
        }
        assert_eq!(wire.get("title").and_then(|v| v.as_str()), Some("Bike"));
        Ok(())
    }

    #[test]
    fn sample_timestamp_parses() {
        let parsed = time::OffsetDateTime::parse(
            &sample_project().timestamp,
            &time::format_description::well_known::Rfc3339,
        );
        assert_eq!(parsed.ok(), Some(datetime!(2026-07-07 14:30:00 UTC)));
    }
}
