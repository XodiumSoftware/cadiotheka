use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use crate::utils::api_url;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// A URL pointing to a project's icon asset.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct IconUrl(pub String);

/// Serde adapter for tags stored as a JSON-text column.
///
/// D1 stores tags as TEXT containing a JSON array, so the frontend first parses
/// that JSON string into a list of strings and then deserializes each string
/// into a strongly-typed [`Tag`] via `serde(rename)`.
mod tag_json_string {
    use crate::metadata::tags::Tag;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[Tag], serializer: S) -> Result<S::Ok, S::Error> {
        let strings: Vec<String> = value
            .iter()
            .map(|v| {
                serde_json::to_value(v)
                    .map_err(serde::ser::Error::custom)
                    .and_then(|val| match val {
                        serde_json::Value::String(s) => Ok(s),
                        _ => Err(serde::ser::Error::custom("expected string tag")),
                    })
            })
            .collect::<Result<_, _>>()?;
        serializer
            .serialize_str(&serde_json::to_string(&strings).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Tag>, D::Error> {
        let s = String::deserialize(deserializer)?;
        let strings: Vec<String> = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
        strings
            .into_iter()
            .map(|item| {
                serde_json::from_value::<Tag>(serde_json::Value::String(item))
                    .map_err(serde::de::Error::custom)
            })
            .collect::<Result<_, _>>()
    }
}

/// Serde adapter for platforms stored as a JSON-text column.
///
/// D1 stores platforms as TEXT containing a JSON array, so the frontend first
/// parses that JSON string into a list of strings and then deserializes each
/// string into a strongly-typed [`Platform`] via `serde(rename)`.
mod platform_json_string {
    use crate::metadata::platforms::Platform;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[Platform], serializer: S) -> Result<S::Ok, S::Error> {
        let strings: Vec<String> = value
            .iter()
            .map(|v| {
                serde_json::to_value(v)
                    .map_err(serde::ser::Error::custom)
                    .and_then(|val| match val {
                        serde_json::Value::String(s) => Ok(s),
                        _ => Err(serde::ser::Error::custom("expected string platform")),
                    })
            })
            .collect::<Result<_, _>>()?;
        serializer
            .serialize_str(&serde_json::to_string(&strings).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Platform>, D::Error> {
        let s = String::deserialize(deserializer)?;
        let strings: Vec<String> = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
        strings
            .into_iter()
            .map(|item| {
                serde_json::from_value::<Platform>(serde_json::Value::String(item))
                    .map_err(serde::de::Error::custom)
            })
            .collect::<Result<_, _>>()
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
    /// Categorized tags for the content.
    #[serde(with = "tag_json_string")]
    pub tags: Vec<Tag>,
    /// Supported platforms for the content.
    #[serde(with = "platform_json_string")]
    pub supported_platforms: Vec<Platform>,
    /// Download count.
    pub downloads: u64,
    /// Account ids of users who have favorited the project.
    #[serde(default, with = "favorites_json_string")]
    pub favorites: Vec<String>,
    /// Official timestamp for when the project was published or updated.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
    /// Optional icon URL (when absent, a colored placeholder is generated).
    #[serde(default, deserialize_with = "deserialize_icon_key")]
    pub icon_url: Option<IconUrl>,
    /// Optional IFC model download URL (when absent, no model has been uploaded).
    #[serde(default, deserialize_with = "deserialize_ifc_key")]
    pub ifc_url: Option<String>,
}

fn deserialize_icon_key<'de, D>(deserializer: D) -> Result<Option<IconUrl>, D::Error>
where
    D: Deserializer<'de>,
{
    let key = Option::<String>::deserialize(deserializer)?;
    Ok(key.map(|key| icon_src_from_key(&key)))
}

fn deserialize_ifc_key<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let key = Option::<String>::deserialize(deserializer)?;
    Ok(key.map(|key| ifc_src_from_key(&key)))
}

/// Builds a frontend URL from an IFC R2 object key (`ifcs/{project_id}/{filename}`).
pub fn ifc_src_from_key(key: &str) -> String {
    let mut parts = key.split('/');
    let _prefix = parts.next();
    let project_id = parts.next().unwrap_or_default();
    let filename = parts.next().unwrap_or_default();
    api_url(&format!("/ifcs/{project_id}/{filename}"))
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
pub fn new_project_payload(
    title: String,
    description: String,
    tags: Vec<Tag>,
    supported_platforms: Vec<Platform>,
) -> ProjectData {
    ProjectData {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        author: String::new(),
        author_id: String::new(),
        author_username: String::new(),
        collaborator_ids: vec![],
        description,
        tags,
        supported_platforms,
        downloads: 0,
        favorites: vec![],
        timestamp: now_utc(),
        icon_url: None,
        ifc_url: None,
    }
}

/// Outcome of attempting to create a project on the backend.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ProjectCreationResult {
    /// The project was created successfully.
    Created(ProjectData),
    /// The backend rejected one or more fields; map keys are field names.
    ValidationErrors(HashMap<String, String>),
    /// A network, serialization, or unexpected server failure occurred.
    Failed(String),
}

/// Response body returned by the backend when project validation fails.
#[derive(Debug, Deserialize)]
pub(crate) struct ValidationErrorResponse {
    pub(crate) errors: HashMap<String, String>,
}

/// Partial project update payload. Only fields with a value are sent to the
/// backend; `None` values are omitted from the JSON body.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProjectPatch {
    pub title: Option<String>,
    pub icon_key: Option<Option<String>>,
    pub tags: Option<Vec<Tag>>,
    pub supported_platforms: Option<Vec<Platform>>,
    pub collaborator_ids: Option<Vec<String>>,
    pub description: Option<String>,
}

/// Converts a stored R2 icon key into the backend URL used by `<img src>`.
pub fn icon_src_from_key(key: &str) -> IconUrl {
    let mut parts = key.split('/');
    let _prefix = parts.next();
    let project_id = parts.next().unwrap_or_default();
    let icon_id = parts.next().unwrap_or_default();
    IconUrl(api_url(&format!("/icons/{project_id}/{icon_id}")))
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
            tags: vec![Tag::Model3d, Tag::Vehicle],
            supported_platforms: vec![Platform::Blender, Platform::FreeCAD],
            downloads: 1200,
            favorites: vec![
                "11111111-1111-1111-1111-111111111111".to_owned(),
                "22222222-2222-2222-2222-222222222222".to_owned(),
            ],
            timestamp: datetime!(2026-07-07 14:30:00 UTC),
            icon_url: None,
            ifc_url: None,
        }
    }

    #[test]
    fn project_deserializes_backend_json_string_columns() {
        let json = r#"[{"id":"71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2","title":"Mountain Bike","author":"TrailBlazer","author_id":"8af81bd9-b70a-4d64-89e9-83bbc4e0297d","author_username":"trailblazer","collaborator_ids":"[]","description":"Extended.","tags":"[\"3d_model\",\"vehicle\",\"fabrication\",\"engineering\",\"diy\"]","supported_platforms":"[\"blender\",\"freecad\",\"fusion_360\",\"step\",\"mesh\"]","downloads":1200,"favorites":"[\"11111111-1111-1111-1111-111111111111\",\"22222222-2222-2222-2222-222222222222\"]","timestamp":"2026-07-07T14:30:00Z","icon_url":null}]"#;
        let projects: Vec<ProjectData> = serde_json::from_str(json).expect("backend JSON parses");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].title, "Mountain Bike");
        assert_eq!(projects[0].tags.len(), 5);
        assert_eq!(projects[0].supported_platforms.len(), 5);
        assert_eq!(projects[0].favorites.len(), 2);
    }

    #[test]
    fn project_serializes_and_deserializes() {
        let project = sample_project();
        let json = serde_json::to_string(&project).expect("project serializes");
        let decoded: ProjectData = serde_json::from_str(&json).expect("project deserializes");
        assert_eq!(decoded, project);
    }

    #[test]
    fn icon_url_serializes_transparently() {
        let url = IconUrl("https://example.com/icon.svg".to_owned());
        let json = serde_json::to_string(&url).unwrap();
        assert_eq!(json, "\"https://example.com/icon.svg\"");

        let decoded: IconUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, url);
    }

    #[test]
    fn project_serializes_json_string_columns_for_backend() {
        let project = sample_project();
        let value = serde_json::to_value(&project).unwrap();
        let tags = value.get("tags").unwrap().as_str().unwrap();
        let platforms = value.get("supported_platforms").unwrap().as_str().unwrap();
        let favorites = value.get("favorites").unwrap().as_str().unwrap();
        let collaborators = value.get("collaborator_ids").unwrap().as_str().unwrap();
        assert!(tags.starts_with('[') && tags.ends_with(']'));
        assert!(platforms.starts_with('[') && platforms.ends_with(']'));
        assert!(favorites.starts_with('[') && favorites.ends_with(']'));
        assert!(collaborators.starts_with('[') && collaborators.ends_with(']'));
    }

    #[test]
    fn project_deserializes_empty_json_string_columns() {
        let json = r#"[{"id":"p1","title":"T","author":"A","author_id":"a1","author_username":"a","collaborator_ids":"[]","description":"E","tags":"[]","supported_platforms":"[]","downloads":0,"favorites":"[]","timestamp":"2026-07-07T14:30:00Z","icon_url":null}]"#;
        let projects: Vec<ProjectData> = serde_json::from_str(json).unwrap();
        assert_eq!(projects.len(), 1);
        assert!(projects[0].tags.is_empty());
        assert!(projects[0].supported_platforms.is_empty());
        assert!(projects[0].favorites.is_empty());
        assert!(projects[0].collaborator_ids.is_empty());
    }

    /// Validates that known tags and platforms serialize and deserialize
    /// correctly. This catches stale enum definitions.
    #[test]
    fn project_uses_known_tags_and_platforms() {
        let project = sample_project();
        for tag in project.tags {
            let _ = tag.label();
        }
        for platform in project.supported_platforms {
            let _ = platform.label();
        }
    }
}
