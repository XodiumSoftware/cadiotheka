pub use crate::metadata::version_state::VersionState;
use crate::utils::api_url;
use serde::{Deserialize, Serialize};

/// Serde adapter for a JSON-text column holding an array of strings.
///
/// D1 stores tags, favorites, and collaborators as TEXT containing a JSON array,
/// so the frontend parses that JSON string into a `Vec<String>`. Tags store
/// the wire ids of records in the `tags` table; their labels and colors are
/// resolved from metadata.
mod string_array_json {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[String], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<String>, D::Error> {
        let s = String::deserialize(deserializer)?;
        serde_json::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Serde adapter for favorites stored as a JSON-text column.
mod favorites_json_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[String], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<String>, D::Error> {
        let s = String::deserialize(deserializer)?;
        serde_json::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// A single IFC file version attached to a project.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    /// Semantic version string for this release (e.g. "1.0.0").
    #[serde(default)]
    pub version: String,
    /// Number of times this version has been downloaded.
    #[serde(default)]
    pub downloads: i64,
}

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

/// Data displayed on a project card.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProjectData {
    /// Unique project identifier.
    pub id: String,
    /// Project title.
    pub title: String,
    /// Author or creator display name.
    pub author: String,
    /// Author account identifier.
    pub author_id: String,
    /// Author username (used for `@author:` filtering and links).
    pub author_username: String,
    /// Account ids of credited collaborators for this project.
    #[serde(default, with = "favorites_json_string")]
    pub collaborator_ids: Vec<String>,
    /// Extended markdown description shown in the project detail modal.
    #[serde(default)]
    pub description: String,
    /// Wire ids of the tags categorizing this content.
    #[serde(with = "string_array_json")]
    pub tags: Vec<String>,
    /// Download count.
    pub downloads: u64,
    /// Account ids of users who have favorited the project.
    #[serde(default, with = "favorites_json_string")]
    pub favorites: Vec<String>,
    /// Official timestamp for when the project was published or updated.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
    /// IFC file versions attached to the project.
    #[serde(default)]
    pub versions: Vec<ProjectVersion>,
}

/// Builds the frontend download URL for an IFC version using its own id and filename.
///
/// The URL is `/data/ifcs/{version_id}/{filename}`, which the backend resolves to the
/// actual R2 object key stored on the version row. This avoids assuming the second
/// segment of `ifc_key` is the version id, which is not true for legacy migrations.
pub fn ifc_download_url(version: &ProjectVersion) -> String {
    api_url(&format!(
        "/ifcs/{version_id}/{filename}",
        version_id = version.id,
        filename = version.filename
    ))
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
    let millis = js_sys::Date::now();
    time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds_f64(millis / 1_000.0)
}

/// Creates a new project payload for submission to the backend.
///
/// The backend fills in `author`, `author_id`, and `downloads`,
/// so this function generates the remaining fields and leaves the computed
/// ones empty or zeroed.
pub fn new_project_payload(title: String, description: String, tags: Vec<String>) -> ProjectData {
    ProjectData {
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
        timestamp: now_utc(),
        versions: vec![],
    }
}

/// Outcome of attempting to create a project on the backend.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ProjectCreationResult {
    /// The project was created successfully.
    Created(ProjectData),
    /// The backend rejected one or more fields; map keys are field names.
    ValidationErrors(std::collections::HashMap<String, String>),
    /// A network, serialization, or unexpected server failure occurred.
    Failed(String),
}

/// Response body returned by the backend when project validation fails.
#[derive(Debug, Deserialize)]
pub(crate) struct ValidationErrorResponse {
    pub(crate) errors: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn sample_project() -> ProjectData {
        ProjectData {
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
            timestamp: datetime!(2026-07-07 14:30:00 UTC),
            versions: vec![],
        }
    }

    #[test]
    fn latest_visible_ifc_url_prefers_first_public_version() {
        let versions = vec![
            ProjectVersion {
                id: "v1".to_owned(),
                project_id: "p1".to_owned(),
                filename: "a.ifc".to_owned(),
                ifc_key: "ifcs/v1/a.ifc".to_owned(),
                state: VersionState::Undefined,
                created_at: "2026-01-01T00:00:00Z".to_owned(),
                file_size: 1_024,
                version: "1.0.0".to_owned(),
                downloads: 0,
            },
            ProjectVersion {
                id: "v2".to_owned(),
                project_id: "p1".to_owned(),
                filename: "b.ifc".to_owned(),
                ifc_key: "ifcs/v2/b.ifc".to_owned(),
                state: VersionState::Stable,
                created_at: "2026-01-02T00:00:00Z".to_owned(),
                file_size: 2_097_152,
                version: "1.1.0".to_owned(),
                downloads: 42,
            },
        ];
        assert_eq!(
            latest_visible_ifc_url(&versions),
            Some(api_url("/ifcs/v2/b.ifc"))
        );
    }

    #[test]
    fn latest_visible_ifc_url_returns_none_when_all_undefined() {
        let versions = vec![ProjectVersion {
            id: "v1".to_owned(),
            project_id: "p1".to_owned(),
            filename: "a.ifc".to_owned(),
            ifc_key: "ifcs/v1/a.ifc".to_owned(),
            state: VersionState::Undefined,
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            file_size: 0,
            version: "1.0.0".to_owned(),
            downloads: 0,
        }];
        assert_eq!(latest_visible_ifc_url(&versions), None);
    }

    #[test]
    fn project_deserializes_backend_json_string_columns() -> Result<(), serde_json::Error> {
        let json = r#"[{"id":"71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2","title":"Mountain Bike","author":"TrailBlazer","author_id":"8af81bd9-b70a-4d64-89e9-83bbc4e0297d","author_username":"trailblazer","collaborator_ids":"[]","description":"Extended.","tags":"[\"3d_model\",\"vehicle\",\"fabrication\",\"engineering\",\"diy\"]","downloads":1200,"favorites":"[\"11111111-1111-1111-1111-111111111111\",\"22222222-2222-2222-2222-222222222222\"]","timestamp":"2026-07-07T14:30:00Z"}]"#;
        let projects: Vec<ProjectData> = serde_json::from_str(json)?;
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
        let decoded: ProjectData = serde_json::from_str(&json)?;
        assert_eq!(decoded, project);
        Ok(())
    }

    #[test]
    fn project_serializes_json_string_columns_for_backend() -> Result<(), serde_json::Error> {
        let project = sample_project();
        let value = serde_json::to_value(&project)?;
        let tags = value
            .get("tags")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let favorites = value
            .get("favorites")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let collaborators = value
            .get("collaborator_ids")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert!(tags.starts_with('[') && tags.ends_with(']'));
        assert!(favorites.starts_with('[') && favorites.ends_with(']'));
        assert!(collaborators.starts_with('[') && collaborators.ends_with(']'));
        Ok(())
    }

    #[test]
    fn project_deserializes_empty_json_string_columns() -> Result<(), serde_json::Error> {
        let json = r#"[{"id":"p1","title":"T","author":"A","author_id":"a1","author_username":"a","collaborator_ids":"[]","description":"E","tags":"[]","downloads":0,"favorites":"[]","timestamp":"2026-07-07T14:30:00Z"}]"#;
        let projects: Vec<ProjectData> = serde_json::from_str(json)?;
        assert_eq!(projects.len(), 1);
        assert!(projects[0].tags.is_empty());
        assert!(projects[0].favorites.is_empty());
        assert!(projects[0].collaborator_ids.is_empty());
        Ok(())
    }

    /// Tags are stored as wire-id strings resolved against the hardcoded enum
    /// in `frontend/src/metadata/tags.rs`.
    #[test]
    fn ifc_download_url_uses_version_id_and_filename() {
        let version = ProjectVersion {
            id: "vid-123".to_owned(),
            project_id: "p1".to_owned(),
            filename: "model.ifc".to_owned(),
            ifc_key: "ifcs/legacy-project-id/model.ifc".to_owned(),
            state: VersionState::Stable,
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            file_size: 0,
            version: "1.0.0".to_owned(),
            downloads: 0,
        };
        assert_eq!(
            ifc_download_url(&version),
            api_url("/ifcs/vid-123/model.ifc")
        );
    }

    #[test]
    fn project_uses_known_tags() {
        let project = sample_project();
        assert_eq!(project.tags, vec!["3d_model", "vehicle"]);
    }
}
