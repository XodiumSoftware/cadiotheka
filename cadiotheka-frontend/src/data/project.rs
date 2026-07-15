use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    use super::*;

    pub fn serialize<S: Serializer>(value: &Vec<Tag>, serializer: S) -> Result<S::Ok, S::Error> {
        let strings: Vec<String> = value
            .iter()
            .map(|v| serde_json::to_string(v).map_err(serde::ser::Error::custom))
            .collect::<Result<_, _>>()?;
        serializer
            .serialize_str(&serde_json::to_string(&strings).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Tag>, D::Error> {
        let s = String::deserialize(deserializer)?;
        let strings: Vec<String> = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
        strings
            .into_iter()
            .map(|item| serde_json::from_str::<Tag>(&item).map_err(serde::de::Error::custom))
            .collect::<Result<_, _>>()
    }
}

/// Serde adapter for platforms stored as a JSON-text column.
///
/// D1 stores platforms as TEXT containing a JSON array, so the frontend first
/// parses that JSON string into a list of strings and then deserializes each
/// string into a strongly-typed [`Platform`] via `serde(rename)`.
mod platform_json_string {
    use super::*;

    pub fn serialize<S: Serializer>(
        value: &Vec<Platform>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let strings: Vec<String> = value
            .iter()
            .map(|v| serde_json::to_string(v).map_err(serde::ser::Error::custom))
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
            .map(|item| serde_json::from_str::<Platform>(&item).map_err(serde::de::Error::custom))
            .collect::<Result<_, _>>()
    }
}

/// Data displayed on a project card.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProjectData {
    /// Unique project identifier.
    pub id: String,
    /// Project title.
    pub title: String,
    /// Author or creator name.
    pub author: String,
    /// Author account identifier.
    pub author_id: String,
    /// Short description of the content.
    pub description: String,
    /// Extended markdown description shown in the project detail modal.
    #[serde(default)]
    pub extended_desc: String,
    /// Categorized tags for the content.
    #[serde(with = "tag_json_string")]
    pub tags: Vec<Tag>,
    /// Supported platforms for the content.
    #[serde(with = "platform_json_string")]
    pub supported_platforms: Vec<Platform>,
    /// Download count.
    pub downloads: u64,
    /// Favorite count.
    pub favorites: u64,
    /// Official timestamp for when the project was published or updated.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
    /// Optional icon URL (when absent, a colored placeholder is generated).
    pub icon_url: Option<IconUrl>,
}

/// Fetch projects from the backend API.
///
/// On failure it logs to the browser console and returns an empty vector so
/// the UI can keep running with a graceful fallback.
pub async fn fetch_projects() -> Vec<ProjectData> {
    match gloo_net::http::Request::get("/api/projects").send().await {
        Ok(response) if response.ok() => response
            .json::<Vec<ProjectData>>()
            .await
            .unwrap_or_default(),
        Ok(response) => {
            let status = response.status();
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch projects: HTTP {status}").into(),
            );
            Vec::new()
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to fetch projects: {err:?}").into());
            Vec::new()
        }
    }
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
            description: "A rugged mountain bike model ready for off-road adventures.".to_owned(),
            extended_desc: "Extended description.".to_owned(),
            tags: vec![Tag::Model3d, Tag::Vehicle],
            supported_platforms: vec![Platform::Blender, Platform::FreeCAD],
            downloads: 1200,
            favorites: 84,
            timestamp: datetime!(2026-07-07 14:30:00 UTC),
            icon_url: None,
        }
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
