use serde::{Deserialize, Serialize};
use worker::*;

use crate::DB_BINDING;
use crate::ICONS_R2_BINDING;
use crate::api::accounts::Account;
use crate::api::session::require_account;
use crate::utils::js_option;

const SELECT_PROJECT_COLUMNS: &str = "SELECT id, title, author, author_id, author_username, collaborator_ids, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url FROM projects";

/// Maximum allowed length for a project title.
const MAX_TITLE_LENGTH: usize = 100;
/// Maximum allowed length for a project short description.
const MAX_DESCRIPTION_LENGTH: usize = 500;
/// Maximum allowed length for a project icon key stored in D1.
const MAX_ICON_KEY_LENGTH: usize = 200;
/// Maximum allowed length for a project's extended markdown description.
const MAX_EXTENDED_DESC_LENGTH: usize = 5000;
/// Maximum allowed size for an uploaded project icon, in bytes.
const MAX_ICON_SIZE_BYTES: usize = 5 * 1024 * 1024; // 5 MiB

/// Validates the project payload and returns an error message when a field
/// exceeds its allowed length. The route handler turns this message into a
/// `400 Bad Request` response.
fn validate_project_payload(payload: &ProjectPayload) -> std::result::Result<(), &'static str> {
    if payload.title.len() > MAX_TITLE_LENGTH {
        return Err("Title must be 100 characters or fewer");
    }
    if payload.description.len() > MAX_DESCRIPTION_LENGTH {
        return Err("Description must be 500 characters or fewer");
    }
    Ok(())
}

/// A Cadiotheka project stored in D1.
#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub author_username: String,
    #[serde(with = "json_string")]
    pub collaborator_ids: Vec<String>,
    pub description: String,
    pub extended_desc: String,
    #[serde(with = "json_string")]
    pub tags: Vec<String>,
    #[serde(with = "json_string")]
    pub supported_platforms: Vec<String>,
    pub downloads: u64,
    #[serde(with = "json_string")]
    pub favorites: Vec<String>,
    pub timestamp: String,
    pub icon_url: Option<String>,
}

/// Payload used to create or update a project.
#[derive(Deserialize, Debug)]
pub struct ProjectPayload {
    pub id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub author_username: String,
    #[serde(with = "json_string")]
    pub collaborator_ids: Vec<String>,
    pub description: String,
    pub extended_desc: String,
    #[serde(with = "json_string")]
    pub tags: Vec<String>,
    #[serde(with = "json_string")]
    pub supported_platforms: Vec<String>,
    pub downloads: u64,
    #[serde(with = "json_string")]
    pub favorites: Vec<String>,
    pub timestamp: String,
    pub icon_url: Option<String>,
}

/// Serde adapter that stores a `Vec<String>` as a single JSON string column.
///
/// D1 stores tags and platforms as TEXT containing a JSON array, so we serialize
/// to a JSON string on the way in and parse that JSON string on the way out.
mod json_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<String>, D::Error> {
        let s = String::deserialize(deserializer)?;
        serde_json::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Returns the D1 database binding configured for this worker.
fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1(DB_BINDING)
}

/// Responds with a JSON array of all projects.
pub async fn list_projects(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let result = db(&ctx)?.prepare(SELECT_PROJECT_COLUMNS).all().await?;
    let projects: Vec<Project> = result.results::<Project>()?;
    Response::from_json(&projects)
}

/// Responds with the project matching the `:id` path parameter, or 404 if not found.
pub async fn read_project(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    match fetch_project(&ctx, &id).await? {
        Some(project) => Response::from_json(&project),
        None => Response::error("Not found", 404),
    }
}

/// Creates a new project from the request body, attributing it to the
/// authenticated user.
pub async fn create_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let mut payload: ProjectPayload = req.json().await?;
    if let Err(msg) = validate_project_payload(&payload) {
        return Response::error(msg, 400);
    }
    payload.author_id = account.id;
    payload.author = account.display_name;
    payload.author_username = account.username;
    let project_id = payload.id.clone();

    let tags = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let platforms =
        serde_json::to_string(&payload.supported_platforms).unwrap_or_else(|_| "[]".to_string());
    let favorites = serde_json::to_string(&payload.favorites).unwrap_or_else(|_| "[]".to_string());
    let collaborator_ids =
        serde_json::to_string(&payload.collaborator_ids).unwrap_or_else(|_| "[]".to_string());

    db(&ctx)?
        .prepare(
            "INSERT INTO projects (id, title, author, author_id, author_username, collaborator_ids, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        )
        .bind(&[
            payload.id.into(),
            payload.title.into(),
            payload.author.into(),
            payload.author_id.into(),
            payload.author_username.into(),
            collaborator_ids.into(),
            payload.description.into(),
            payload.extended_desc.into(),
            tags.into(),
            platforms.into(),
            (payload.downloads as f64).into(),
            favorites.into(),
            payload.timestamp.into(),
            js_option(payload.icon_url),
        ])?
        .run()
        .await?;

    let created = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("created project not found".into()))?;
    Response::from_json(&created)
}

/// Partial payload for patching a project. All fields are optional; only the
/// provided fields are updated.
#[derive(Deserialize, Debug)]
pub struct ProjectPatch {
    title: Option<String>,
    icon_key: Option<Option<String>>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    supported_platforms: Option<Vec<String>>,
    collaborator_ids: Option<Vec<String>>,
    extended_desc: Option<String>,
}

/// Partially updates an existing project, identified by the `:id` path parameter.
/// Only the project owner or an admin may edit it.
pub async fn patch_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return Response::error("Forbidden", 403);
    }

    let patch: ProjectPatch = req.json().await?;
    if let Some(title) = patch.title {
        if title.len() > MAX_TITLE_LENGTH {
            return Response::error("Title must be 100 characters or fewer", 400);
        }
        db(&ctx)?
            .prepare("UPDATE projects SET title = ?1 WHERE id = ?2")
            .bind(&[title.into(), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(icon_key) = patch.icon_key {
        if let Some(ref key) = icon_key {
            if key.len() > MAX_ICON_KEY_LENGTH {
                return Response::error("Icon key must be 200 characters or fewer", 400);
            }
            if !key.starts_with("icons/") {
                return Response::error("Invalid icon key", 400);
            }
        }
        db(&ctx)?
            .prepare("UPDATE projects SET icon_url = ?1 WHERE id = ?2")
            .bind(&[js_option(icon_key), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(description) = patch.description {
        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Response::error("Description must be 500 characters or fewer", 400);
        }
        db(&ctx)?
            .prepare("UPDATE projects SET description = ?1 WHERE id = ?2")
            .bind(&[description.into(), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(tags) = patch.tags {
        let tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        db(&ctx)?
            .prepare("UPDATE projects SET tags = ?1 WHERE id = ?2")
            .bind(&[tags.into(), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(supported_platforms) = patch.supported_platforms {
        let supported_platforms =
            serde_json::to_string(&supported_platforms).unwrap_or_else(|_| "[]".to_string());
        db(&ctx)?
            .prepare("UPDATE projects SET supported_platforms = ?1 WHERE id = ?2")
            .bind(&[supported_platforms.into(), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(collaborator_ids) = patch.collaborator_ids {
        let collaborator_ids =
            serde_json::to_string(&collaborator_ids).unwrap_or_else(|_| "[]".to_string());
        db(&ctx)?
            .prepare("UPDATE projects SET collaborator_ids = ?1 WHERE id = ?2")
            .bind(&[collaborator_ids.into(), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(extended_desc) = patch.extended_desc {
        if extended_desc.len() > MAX_EXTENDED_DESC_LENGTH {
            return Response::error("Extended description must be 5000 characters or fewer", 400);
        }
        db(&ctx)?
            .prepare("UPDATE projects SET extended_desc = ?1 WHERE id = ?2")
            .bind(&[extended_desc.into(), id.into()])?
            .run()
            .await?;
    }

    Response::empty()
}

/// Replaces an existing project, identified by the `:id` path parameter.
pub async fn update_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return Response::error("Forbidden", 403);
    }

    let mut payload: ProjectPayload = req.json().await?;
    if let Err(msg) = validate_project_payload(&payload) {
        return Response::error(msg, 400);
    }
    payload.author_id = project.author_id;
    payload.author = project.author;
    payload.author_username = project.author_username;
    let tags = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let platforms =
        serde_json::to_string(&payload.supported_platforms).unwrap_or_else(|_| "[]".to_string());
    payload.collaborator_ids = project.collaborator_ids.clone();
    let favorites = serde_json::to_string(&project.favorites).unwrap_or_else(|_| "[]".to_string());
    let collaborator_ids =
        serde_json::to_string(&payload.collaborator_ids).unwrap_or_else(|_| "[]".to_string());

    db(&ctx)?
        .prepare(
            "UPDATE projects \
             SET title = ?1, author = ?2, author_id = ?3, author_username = ?4, collaborator_ids = ?5, description = ?6, extended_desc = ?7, tags = ?8, supported_platforms = ?9, downloads = ?10, favorites = ?11, timestamp = ?12, icon_url = ?13 \
             WHERE id = ?14",
        )
        .bind(&[
            payload.title.into(),
            payload.author.into(),
            payload.author_id.into(),
            payload.author_username.into(),
            collaborator_ids.into(),
            payload.description.into(),
            payload.extended_desc.into(),
            tags.into(),
            platforms.into(),
            (payload.downloads as f64).into(),
            favorites.into(),
            payload.timestamp.into(),
            js_option(payload.icon_url),
            id.into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Deletes the project identified by the `:id` path parameter.
pub async fn delete_project(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return Response::error("Forbidden", 403);
    }

    db(&ctx)?
        .prepare("DELETE FROM projects WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;

    if let Some(icon_key) = project.icon_url
        && let Err(err) = icons_bucket(&ctx)?.delete(&icon_key).await
    {
        console_log!("Failed to delete icon {} for project: {:?}", icon_key, err);
    }

    Response::empty()
}

/// Returns the R2 bucket configured for project icons.
fn icons_bucket(ctx: &RouteContext<()>) -> Result<Bucket> {
    ctx.env.bucket(ICONS_R2_BINDING)
}

/// Returns the MIME type for an icon based on its magic bytes.
fn icon_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && bytes[0..4] == *b"RIFF" && bytes[8..12] == *b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Handles a multipart upload of a project icon, validates it, stores it in R2,
/// and updates the project's `icon_url` column with the R2 object key.
pub async fn upload_project_icon(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return Response::error("Forbidden", 403);
    }

    let form_data = req.form_data().await?;
    let file = match form_data.get("icon") {
        Some(FormEntry::File(file)) => file,
        _ => {
            return Response::error("missing icon file", 400);
        }
    };

    let bytes = file.bytes().await?;
    if bytes.len() > MAX_ICON_SIZE_BYTES {
        return Response::error("Icon must be 5 MiB or smaller", 413);
    }

    let content_type = icon_content_type(&bytes)
        .ok_or_else(|| worker::Error::RustError("Icon must be PNG, JPEG, or WebP".into()))?;

    let old_key = project.icon_url.clone();
    let key = format!("icons/{id}/icon");
    let http_metadata = HttpMetadata {
        content_type: Some(content_type.to_string()),
        ..Default::default()
    };

    icons_bucket(&ctx)?
        .put(&key, bytes)
        .http_metadata(http_metadata)
        .execute()
        .await?;

    db(&ctx)?
        .prepare("UPDATE projects SET icon_url = ?1 WHERE id = ?2")
        .bind(&[key.clone().into(), id.into()])?
        .run()
        .await?;

    if let Some(old_key) = old_key.filter(|k| k != &key) {
        let _ = icons_bucket(&ctx)?.delete(&old_key).await;
    }

    Response::from_json(&serde_json::json!({ "icon_key": key, "content_type": content_type }))
}

/// Serves an icon from R2 by its object key.
pub async fn serve_icon(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let project_id = ctx.param("project_id").cloned().unwrap_or_default();
    let icon_id = ctx.param("icon_id").cloned().unwrap_or_default();
    if project_id.is_empty() || icon_id.is_empty() {
        return Response::error("Invalid icon key", 400);
    }
    let key = format!("icons/{project_id}/{icon_id}");

    let object = icons_bucket(&ctx)?.get(&key).execute().await?;

    let Some(object) = object else {
        return Response::error("Not found", 404);
    };

    let http_metadata = object.http_metadata();
    let content_type = http_metadata
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let body = object
        .body()
        .ok_or_else(|| worker::Error::RustError("icon object has no body".into()))?;

    let headers = Headers::new();
    headers.set("Content-Type", &content_type)?;
    Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers))
}

/// Whether the given account may edit or delete the project.
fn can_edit_project(account: &Account, project: &Project) -> bool {
    account.role == "admin" || account.id == project.author_id
}

pub async fn toggle_project_favorite(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let mut project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;

    if let Some(index) = project
        .favorites
        .iter()
        .position(|user_id| user_id == &account.id)
    {
        project.favorites.remove(index);
    } else {
        project.favorites.push(account.id.clone());
    }

    let favorites = serde_json::to_string(&project.favorites).unwrap_or_else(|_| "[]".to_string());
    db(&ctx)?
        .prepare("UPDATE projects SET favorites = ?1 WHERE id = ?2")
        .bind(&[favorites.into(), project.id.clone().into()])?
        .run()
        .await?;

    let updated = fetch_project(&ctx, &project.id)
        .await?
        .ok_or_else(|| worker::Error::RustError("updated project not found".into()))?;
    Response::from_json(&updated)
}

/// Fetches a single project by id, returning `None` when no row matches.
async fn fetch_project(ctx: &RouteContext<()>, id: &str) -> Result<Option<Project>> {
    let result = db(ctx)?
        .prepare(format!("{SELECT_PROJECT_COLUMNS} WHERE id = ?1"))
        .bind(&[id.into()])?
        .all()
        .await?;
    let mut projects: Vec<Project> = result.results::<Project>()?;
    Ok(projects.pop())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_account(role: &str) -> Account {
        Account {
            id: "acc-1".into(),
            username: "creator".into(),
            display_name: "Creator".into(),
            email: "creator@example.com".into(),
            role: role.into(),
            bio: "".into(),
            avatar_url: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            verified: 1,
        }
    }

    fn sample_project(author_id: &str) -> Project {
        Project {
            id: "proj-1".into(),
            title: "Sample".into(),
            author: "Author".into(),
            author_id: author_id.into(),
            author_username: "author".into(),
            collaborator_ids: vec![],
            description: "".into(),
            extended_desc: "".into(),
            tags: vec![],
            supported_platforms: vec![],
            downloads: 0,
            favorites: vec![],
            timestamp: "2025-01-01T00:00:00Z".into(),
            icon_url: None,
        }
    }

    fn sample_payload() -> ProjectPayload {
        ProjectPayload {
            id: "proj-1".into(),
            title: "Sample".into(),
            author: "Author".into(),
            author_id: "acc-1".into(),
            author_username: "author".into(),
            collaborator_ids: vec![],
            description: "A short description.".into(),
            extended_desc: "".into(),
            tags: vec![],
            supported_platforms: vec![],
            downloads: 0,
            favorites: vec![],
            timestamp: "2025-01-01T00:00:00Z".into(),
            icon_url: None,
        }
    }

    #[test]
    fn payload_with_valid_title_and_description_passes() {
        assert!(validate_project_payload(&sample_payload()).is_ok());
    }

    #[test]
    fn payload_with_long_title_fails() {
        let mut payload = sample_payload();
        payload.title = "a".repeat(101);
        assert_eq!(
            validate_project_payload(&payload),
            Err("Title must be 100 characters or fewer")
        );
    }

    #[test]
    fn payload_with_long_description_fails() {
        let mut payload = sample_payload();
        payload.description = "a".repeat(501);
        assert_eq!(
            validate_project_payload(&payload),
            Err("Description must be 500 characters or fewer")
        );
    }

    #[test]
    fn owner_can_edit_project() {
        let account = sample_account("creator");
        let project = sample_project(&account.id);
        assert!(can_edit_project(&account, &project));
    }

    #[test]
    fn non_owner_cannot_edit_project() {
        let account = sample_account("creator");
        let project = sample_project("other");
        assert!(!can_edit_project(&account, &project));
    }

    #[test]
    fn admin_can_edit_any_project() {
        let account = sample_account("admin");
        let project = sample_project("other");
        assert!(can_edit_project(&account, &project));
    }
}
