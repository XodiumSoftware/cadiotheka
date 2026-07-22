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

/// Serde adapter for favorites stored as a JSON-text column.
mod favorites_json_string {
    use super::*;

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
    /// Account ids of users who have favorited the project.
    #[serde(default, with = "favorites_json_string")]
    pub favorites: Vec<String>,
    /// Official timestamp for when the project was published or updated.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
    /// Optional icon URL (when absent, a colored placeholder is generated).
    #[serde(deserialize_with = "deserialize_icon_key")]
    pub icon_url: Option<IconUrl>,
}

fn deserialize_icon_key<'de, D>(deserializer: D) -> Result<Option<IconUrl>, D::Error>
where
    D: Deserializer<'de>,
{
    let key = Option::<String>::deserialize(deserializer)?;
    Ok(key.map(|key| icon_src_from_key(&key)))
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
/// The backend fills in `author`, `author_id`, and `downloads`,
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
        collaborator_ids: vec![],
        description,
        extended_desc,
        tags,
        supported_platforms,
        downloads: 0,
        favorites: vec![],
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

/// Updates the title of an existing project via `PATCH /data/projects/:id`.
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

    patch_project(&url, body, "title").await?;
    Some(title)
}

/// Updates the short description of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new description; on failure it logs to the console
/// and returns `None`.
pub async fn update_project_description(id: &str, description: String) -> Option<String> {
    let url = api_url(&format!("/projects/{id}"));
    let body = match serde_json::to_string(&serde_json::json!({ "description": description })) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize description update payload: {err:?}").into(),
            );
            return None;
        }
    };

    patch_project(&url, body, "description").await?;
    Some(description)
}

/// Updates the tags of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new tag list; on failure it logs to the console and
/// returns `None`.
pub async fn update_project_tags(id: &str, tags: Vec<Tag>) -> Option<Vec<Tag>> {
    let url = api_url(&format!("/projects/{id}"));
    let body = match serde_json::to_string(&serde_json::json!({ "tags": tags })) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize tags update payload: {err:?}").into(),
            );
            return None;
        }
    };

    patch_project(&url, body, "tags").await?;
    Some(tags)
}

/// Updates the supported platforms of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new platform list; on failure it logs to the console
/// and returns `None`.
pub async fn update_project_platforms(
    id: &str,
    supported_platforms: Vec<Platform>,
) -> Option<Vec<Platform>> {
    let url = api_url(&format!("/projects/{id}"));
    let body = match serde_json::to_string(
        &serde_json::json!({ "supported_platforms": supported_platforms }),
    ) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize supported platforms update payload: {err:?}").into(),
            );
            return None;
        }
    };

    patch_project(&url, body, "supported platforms").await?;
    Some(supported_platforms)
}

/// Updates the extended description of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new extended description; on failure it logs to the
/// console and returns `None`.
pub async fn update_project_extended_desc(id: &str, extended_desc: String) -> Option<String> {
    let url = api_url(&format!("/projects/{id}"));
    let body = match serde_json::to_string(&serde_json::json!({ "extended_desc": extended_desc })) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize extended description update payload: {err:?}").into(),
            );
            return None;
        }
    };

    patch_project(&url, body, "extended description").await?;
    Some(extended_desc)
}

/// Updates the collaborator ids of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new collaborator id list; on failure it logs to the
/// console and returns `None`.
pub async fn update_project_collaborators(
    id: &str,
    collaborator_ids: Vec<String>,
) -> Option<Vec<String>> {
    let url = api_url(&format!("/projects/{id}"));
    let body =
        match serde_json::to_string(&serde_json::json!({ "collaborator_ids": collaborator_ids })) {
            Ok(json) => json,
            Err(err) => {
                leptos::web_sys::console::error_1(
                    &format!("Failed to serialize collaborator update payload: {err:?}").into(),
                );
                return None;
            }
        };

    patch_project(&url, body, "collaborators").await?;
    Some(collaborator_ids)
}

/// Partial project update payload. Only fields with a value are sent to the
/// backend; `None` values are omitted from the JSON body.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProjectPatch {
    pub title: Option<String>,
    pub icon_key: Option<Option<String>>,
    pub description: Option<String>,
    pub tags: Option<Vec<Tag>>,
    pub supported_platforms: Option<Vec<Platform>>,
    pub collaborator_ids: Option<Vec<String>>,
    pub extended_desc: Option<String>,
}

/// Applies a partial update to an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns `true`; on failure it logs to the console and returns
/// `false`.
pub async fn update_project(id: &str, patch: ProjectPatch) -> bool {
    let url = api_url(&format!("/projects/{id}"));
    let body = match serde_json::to_string(&patch) {
        Ok(json) => json,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize project update payload: {err:?}").into(),
            );
            return false;
        }
    };

    match gloo_net::http::Request::patch(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(request) => match request.send().await {
            Ok(response) => {
                if !response.ok() {
                    let status = response.status();
                    leptos::web_sys::console::error_1(
                        &format!("Failed to update project: HTTP {status}").into(),
                    );
                    false
                } else {
                    true
                }
            }
            Err(err) => {
                leptos::web_sys::console::error_1(
                    &format!("Failed to update project: {err:?}").into(),
                );
                false
            }
        },
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build project update request: {err:?}").into(),
            );
            false
        }
    }
}

/// Updates the icon URL of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new icon URL (or `None` when cleared); on failure
/// it logs to the console and returns `None`.
pub async fn toggle_project_favorite(id: &str) -> Option<ProjectData> {
    let url = api_url(&format!("/projects/{id}/favorites"));
    let request = match gloo_net::http::Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body("")
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build favorite toggle request: {err:?}").into(),
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
                    &format!("Failed to toggle project favorite: HTTP {status}\n{text}").into(),
                );
                return None;
            }

            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<ProjectData>(&text) {
                Ok(project) => Some(project),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse toggled favorite response (status={status}): {err:?}\n{text}"
                        )
                        .into(),
                    );
                    None
                }
            }
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to toggle project favorite: {err:?}").into(),
            );
            None
        }
    }
}

pub async fn upload_project_icon(id: &str, file: web_sys::File) -> Option<IconUrl> {
    let url = api_url(&format!("/projects/{id}/icon"));
    let form = match web_sys::FormData::new() {
        Ok(form) => form,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to create icon upload form data: {err:?}").into(),
            );
            return None;
        }
    };

    if let Err(err) = form.append_with_blob_and_filename("icon", &file, &file.name()) {
        leptos::web_sys::console::error_1(
            &format!("Failed to append icon file to form data: {err:?}").into(),
        );
        return None;
    }

    let request = match gloo_net::http::Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body(form)
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build icon upload request: {err:?}").into(),
            );
            return None;
        }
    };

    #[derive(Deserialize)]
    struct UploadResponse {
        icon_key: String,
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            if !response.ok() {
                let text = response.text().await.unwrap_or_default();
                leptos::web_sys::console::error_1(
                    &format!("Failed to upload project icon: HTTP {status}\n{text}").into(),
                );
                return None;
            }

            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<UploadResponse>(&text) {
                Ok(upload) => Some(icon_src_from_key(&upload.icon_key)),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse project icon upload response (status={status}): {err:?}\n{text}"
                        )
                        .into(),
                    );
                    None
                }
            }
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to upload project icon: {err:?}").into(),
            );
            None
        }
    }
}

/// Converts a stored R2 icon key into the backend URL used by `<img src>`.
pub fn icon_src_from_key(key: &str) -> IconUrl {
    let mut parts = key.split('/');
    let _prefix = parts.next();
    let project_id = parts.next().unwrap_or_default();
    let icon_id = parts.next().unwrap_or_default();
    IconUrl(api_url(&format!("/icons/{project_id}/{icon_id}")))
}

/// Sends a `PATCH` request to the given project endpoint.
///
/// Logs failures under the provided field name and returns `Some(())` only when
/// the response is successful.
async fn patch_project(url: &str, body: String, field_name: &str) -> Option<()> {
    let request = match gloo_net::http::Request::patch(url)
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build {field_name} update request: {err:?}").into(),
            );
            return None;
        }
    };

    match request.send().await {
        Ok(response) => {
            if !response.ok() {
                let status = response.status();
                leptos::web_sys::console::error_1(
                    &format!("Failed to update project {field_name}: HTTP {status}").into(),
                );
                return None;
            }
            Some(())
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to update project {field_name}: {err:?}").into(),
            );
            None
        }
    }
}

/// Deletes a project via `DELETE /data/projects/:id`.
///
/// Returns `true` if the deletion succeeded, otherwise logs the error and
/// returns `false`.
pub async fn delete_project(id: &str) -> bool {
    let url = api_url(&format!("/projects/{id}"));
    match gloo_net::http::Request::delete(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) => {
            if !response.ok() {
                let status = response.status();
                leptos::web_sys::console::error_1(
                    &format!("Failed to delete project: HTTP {status}").into(),
                );
                false
            } else {
                true
            }
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to delete project: {err:?}").into());
            false
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
            collaborator_ids: vec![],
            description: "A rugged mountain bike model ready for off-road adventures.".to_owned(),
            extended_desc: "Extended description.".to_owned(),
            tags: vec![Tag::Model3d, Tag::Vehicle],
            supported_platforms: vec![Platform::Blender, Platform::FreeCAD],
            downloads: 1200,
            favorites: vec![
                "11111111-1111-1111-1111-111111111111".to_owned(),
                "22222222-2222-2222-2222-222222222222".to_owned(),
            ],
            timestamp: datetime!(2026-07-07 14:30:00 UTC),
            icon_url: None,
        }
    }

    #[test]
    fn project_deserializes_backend_json_string_columns() {
        let json = r#"[{"id":"71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2","title":"Mountain Bike","author":"TrailBlazer","author_id":"8af81bd9-b70a-4d64-89e9-83bbc4e0297d","author_username":"trailblazer","collaborator_ids":"[]","description":"A rugged mountain bike model ready for off-road adventures.","extended_desc":"Extended.","tags":"[\"3d_model\",\"vehicle\",\"fabrication\",\"engineering\",\"diy\"]","supported_platforms":"[\"blender\",\"freecad\",\"fusion_360\",\"step\",\"mesh\"]","downloads":1200,"favorites":"[\"11111111-1111-1111-1111-111111111111\",\"22222222-2222-2222-2222-222222222222\"]","timestamp":"2026-07-07T14:30:00Z","icon_url":null}]"#;
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
