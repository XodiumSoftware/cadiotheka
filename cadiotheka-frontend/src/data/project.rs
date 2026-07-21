use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use crate::utils::api_url;
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

    pub fn serialize<S: Serializer>(value: &[Tag], serializer: S) -> Result<S::Ok, S::Error> {
        let strings: Vec<String> = value
            .iter()
            .map(|v| {
                // Emit the raw enum rename value, not a quoted JSON string.
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
    use super::*;

    pub fn serialize<S: Serializer>(value: &[Platform], serializer: S) -> Result<S::Ok, S::Error> {
        let strings: Vec<String> = value
            .iter()
            .map(|v| {
                // Emit the raw enum rename value, not a quoted JSON string.
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

/// Returns the current UTC time using the JavaScript `Date` API.
fn now_utc() -> time::OffsetDateTime {
    let millis = js_sys::Date::now();
    let seconds = (millis / 1_000.0) as i64;
    let nanos = ((millis % 1_000.0) * 1_000_000.0) as i32;
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
        + time::Duration::nanoseconds(nanos.into())
}

/// Creates a new project payload for submission to the backend.
///
/// The backend fills in `author`, `author_id`, `downloads`, and `favorites`,
/// so this function generates the remaining fields and leaves the computed
/// ones empty or zeroed.
pub fn new_project_payload(
    title: String,
    description: String,
    extended_desc: String,
    tags: Vec<Tag>,
    supported_platforms: Vec<Platform>,
) -> ProjectData {
    ProjectData {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        author: String::new(),
        author_id: String::new(),
        author_username: String::new(),
        description,
        extended_desc,
        tags,
        supported_platforms,
        downloads: 0,
        favorites: 0,
        timestamp: now_utc(),
        icon_url: None,
    }
}

/// Submits a new project to the backend API.
///
/// Sends a `POST /data/projects` request with credentials so the session
/// cookie is included. On success the created project is returned; on failure
/// `None` is returned and the error is logged to the browser console.
pub async fn create_project(project: &ProjectData) -> Option<ProjectData> {
    let url = api_url("/projects");
    let body = match serde_json::to_string(project) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize project payload: {err:?}").into(),
            );
            return None;
        }
    };

    let request = match gloo_net::http::Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build project creation request: {err:?}").into(),
            );
            return None;
        }
    };

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            if !response.ok() {
                let text = response.text().await.unwrap_or_default();
                leptos::web_sys::console::error_1(
                    &format!("Failed to create project: HTTP {status}\n{text}").into(),
                );
                return None;
            }

            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<ProjectData>(&text) {
                Ok(data) => Some(data),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse created project response (status={status}): {err:?}\n{text}"
                        )
                        .into(),
                    );
                    None
                }
            }
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to create project: {err:?}").into());
            None
        }
    }
}

/// Updates a single field of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new title; on failure it logs to the console and
/// returns `None`.
pub async fn update_project_title(id: &str, title: String) -> Option<String> {
    let url = api_url(&format!("/projects/{id}"));
    let body = match serde_json::to_string(&serde_json::json!({ "title": title })) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize title update payload: {err:?}").into(),
            );
            return None;
        }
    };

    let request = match gloo_net::http::Request::patch(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build title update request: {err:?}").into(),
            );
            return None;
        }
    };

    match request.send().await {
        Ok(response) => {
            if !response.ok() {
                let status = response.status();
                leptos::web_sys::console::error_1(
                    &format!("Failed to update project title: HTTP {status}").into(),
                );
                return None;
            }
            Some(title)
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to update project title: {err:?}").into(),
            );
            None
        }
    }
}

/// Fetch projects from the backend API.
///
/// On failure it logs to the browser console and returns an empty vector so
/// the UI can keep running with a graceful fallback.
pub async fn fetch_projects() -> Vec<ProjectData> {
    match gloo_net::http::Request::get(&api_url("/projects"))
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<Vec<ProjectData>>(&text) {
                Ok(data) => data,
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to parse projects JSON: {err:?}\n{text}").into(),
                    );
                    Vec::new()
                }
            }
        }
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
            author_username: "trailblazer".to_owned(),
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
    fn project_deserializes_backend_json_string_columns() {
        let json = r#"[{"id":"71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2","title":"Mountain Bike","author":"TrailBlazer","author_id":"8af81bd9-b70a-4d64-89e9-83bbc4e0297d","author_username":"trailblazer","description":"A rugged mountain bike model ready for off-road adventures.","extended_desc":"Extended.","tags":"[\"3d_model\",\"vehicle\",\"fabrication\",\"engineering\",\"diy\"]","supported_platforms":"[\"blender\",\"freecad\",\"fusion_360\",\"step\",\"mesh\"]","downloads":1200,"favorites":84,"timestamp":"2026-07-07T14:30:00Z","icon_url":null}]"#;
        let projects: Vec<ProjectData> = serde_json::from_str(json).expect("backend JSON parses");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].title, "Mountain Bike");
        assert_eq!(projects[0].tags.len(), 5);
        assert_eq!(projects[0].supported_platforms.len(), 5);
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
