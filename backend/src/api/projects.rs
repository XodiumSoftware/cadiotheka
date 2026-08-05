use serde::{Deserialize, Serialize};
use worker::{
    Bucket, D1Database, FormEntry, Headers, HttpMetadata, Request, Response, Result, RouteContext,
    console_log,
};

use crate::DB_BINDING;
use crate::PROJECT_ASSETS_R2_BINDING;
use crate::api::accounts::Account;
use crate::api::session::require_account;
use crate::api::turnstile::verify_turnstile_token;
use crate::utils::{check_rate_limit, error_response, now_utc};
use ifc_lite_export::{GltfOptions, export_glb};

const SELECT_PROJECT_COLUMNS: &str = "SELECT id, title, author, author_id, author_username, collaborator_ids, description, tags, platforms, downloads, favorites, timestamp FROM projects";

/// Maximum allowed length for a project title.
const MAX_TITLE_LENGTH: usize = 100;
/// Maximum allowed length for a project description.
const MAX_DESCRIPTION_LENGTH: usize = 5000;
/// Maximum allowed size for an uploaded project IFC model, in bytes.
const MAX_IFC_SIZE_BYTES: usize = 25 * 1024 * 1024; // 25 MiB

/// A version state for an IFC file.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionState {
    Undefined,
    Alpha,
    Beta,
    Stable,
}

impl VersionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Stable => "stable",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "undefined" => Ok(Self::Undefined),
            "alpha" => Ok(Self::Alpha),
            "beta" => Ok(Self::Beta),
            "stable" => Ok(Self::Stable),
            _ => Err(worker::Error::RustError(format!(
                "invalid version state: {value}"
            ))),
        }
    }
}

/// A single IFC file version attached to a project.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectVersion {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub ifc_key: String,
    pub state: VersionState,
    pub created_at: String,
    pub file_size: i64,
}

/// Payload used to update a version's state.
#[derive(Deserialize, Debug)]
pub struct VersionPatch {
    pub state: String,
}

/// Row shape used when looking up an IFC key by version id.
#[derive(Deserialize)]
struct VersionKey {
    ifc_key: String,
}
/// Validates the project payload and returns a map of field names to error
/// messages. An empty map means the payload is valid.
fn validate_project_payload(payload: &ProjectPayload) -> std::collections::HashMap<String, String> {
    let mut errors = std::collections::HashMap::new();
    if payload.title.len() > MAX_TITLE_LENGTH {
        errors.insert(
            "title".to_string(),
            "Title must be 100 characters or fewer".to_string(),
        );
    }
    errors
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
    #[serde(with = "json_string")]
    pub tags: Vec<String>,
    #[serde(with = "json_string")]
    pub platforms: Vec<String>,
    pub downloads: u64,
    #[serde(with = "json_string")]
    pub favorites: Vec<String>,
    pub timestamp: String,
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
    #[serde(with = "json_string")]
    pub tags: Vec<String>,
    #[serde(with = "json_string")]
    pub platforms: Vec<String>,
    pub downloads: u64,
    #[serde(with = "json_string")]
    pub favorites: Vec<String>,
    pub timestamp: String,
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
        None => error_response("Not found", 404),
    }
}

/// Creates a new project from the request body, attributing it to the
/// authenticated user.
pub async fn create_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "project_create").await? {
        return Ok(response);
    }
    if let Some(response) = verify_turnstile_token(&mut req, &ctx).await? {
        return Ok(response);
    }
    let account = require_account(&req, &ctx).await?;
    let mut payload: ProjectPayload = req.json().await?;
    let validation_errors = validate_project_payload(&payload);
    if !validation_errors.is_empty() {
        let body = serde_json::json!({ "errors": validation_errors });
        return Ok(Response::from_json(&body)?.with_status(400));
    }
    payload.author_id = account.id;
    payload.author = account.display_name;
    payload.author_username = account.username;
    let project_id = payload.id.clone();

    let tags = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let platforms = serde_json::to_string(&payload.platforms).unwrap_or_else(|_| "[]".to_string());
    let favorites = serde_json::to_string(&payload.favorites).unwrap_or_else(|_| "[]".to_string());
    let collaborator_ids =
        serde_json::to_string(&payload.collaborator_ids).unwrap_or_else(|_| "[]".to_string());

    #[allow(clippy::cast_precision_loss)]
    let downloads_value = payload.downloads as f64;

    db(&ctx)?
        .prepare(
            "INSERT INTO projects (id, title, author, author_id, author_username, collaborator_ids, description, tags, platforms, downloads, favorites, timestamp) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(
            &[
                payload.id.into(),
                payload.title.into(),
                payload.author.into(),
                payload.author_id.into(),
                payload.author_username.into(),
                collaborator_ids.into(),
                payload.description.into(),
                tags.into(),
                platforms.into(),
                downloads_value.into(),
                favorites.into(),
                payload.timestamp.into(),
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
#[allow(clippy::option_option)]
pub struct ProjectPatch {
    title: Option<String>,
    tags: Option<Vec<String>>,
    platforms: Option<Vec<String>>,
    collaborator_ids: Option<Vec<String>>,
    description: Option<String>,
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
        return error_response("Forbidden", 403);
    }

    let patch: ProjectPatch = req.json().await?;
    if let Some(title) = patch.title {
        if title.len() > MAX_TITLE_LENGTH {
            return error_response("Title must be 100 characters or fewer", 400);
        }
        db(&ctx)?
            .prepare("UPDATE projects SET title = ?1 WHERE id = ?2")
            .bind(&[title.into(), id.clone().into()])?
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

    if let Some(platforms) = patch.platforms {
        let platforms = serde_json::to_string(&platforms).unwrap_or_else(|_| "[]".to_string());
        db(&ctx)?
            .prepare("UPDATE projects SET platforms = ?1 WHERE id = ?2")
            .bind(&[platforms.into(), id.clone().into()])?
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

    if let Some(description) = patch.description {
        if description.len() > MAX_DESCRIPTION_LENGTH {
            return error_response("Description must be 5000 characters or fewer", 400);
        }
        db(&ctx)?
            .prepare("UPDATE projects SET description = ?1 WHERE id = ?2")
            .bind(&[description.into(), id.into()])?
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
        return error_response("Forbidden", 403);
    }

    let mut payload: ProjectPayload = req.json().await?;
    let validation_errors = validate_project_payload(&payload);
    if !validation_errors.is_empty() {
        let body = serde_json::json!({ "errors": validation_errors });
        return Ok(Response::from_json(&body)?.with_status(400));
    }
    payload.author_id = project.author_id;
    payload.author = project.author;
    payload.author_username = project.author_username;
    let tags = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let platforms = serde_json::to_string(&payload.platforms).unwrap_or_else(|_| "[]".to_string());
    payload.collaborator_ids = project.collaborator_ids.clone();
    let favorites = serde_json::to_string(&project.favorites).unwrap_or_else(|_| "[]".to_string());
    let collaborator_ids =
        serde_json::to_string(&payload.collaborator_ids).unwrap_or_else(|_| "[]".to_string());

    #[allow(clippy::cast_precision_loss)]
    let downloads_value = payload.downloads as f64;

    db(&ctx)?
        .prepare(
            "UPDATE projects \
             SET title = ?1, author = ?2, author_id = ?3, author_username = ?4, collaborator_ids = ?5, description = ?6, tags = ?7, platforms = ?8, downloads = ?9, favorites = ?10, timestamp = ?11 \
             WHERE id = ?12",
        )
        .bind(
            &[
                payload.title.into(),
                payload.author.into(),
                payload.author_id.into(),
                payload.author_username.into(),
                collaborator_ids.into(),
                payload.description.into(),
                tags.into(),
                platforms.into(),
                downloads_value.into(),
                favorites.into(),
                payload.timestamp.into(),
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
        return error_response("Forbidden", 403);
    }

    db(&ctx)?
        .prepare("DELETE FROM projects WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;

    Response::empty()
}

/// Returns the R2 bucket used for project IFC models.
fn ifcs_bucket(ctx: &RouteContext<()>) -> Result<Bucket> {
    ctx.env.bucket(PROJECT_ASSETS_R2_BINDING)
}

/// Handles a multipart upload of a project IFC model, stores it in R2,
/// and creates a new `project_versions` row with state `undefined`.
pub async fn upload_project_ifc(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "ifc_upload").await? {
        return Ok(response);
    }
    let account = require_account(&req, &ctx).await?;
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return error_response("Forbidden", 403);
    }

    let form_data = req.form_data().await?;
    let Some(FormEntry::File(file)) = form_data.get("ifc") else {
        return error_response("missing IFC file", 400);
    };

    let bytes = file.bytes().await?;
    #[allow(clippy::cast_possible_wrap)]
    let file_size_i64 = bytes.len() as i64;
    #[allow(clippy::cast_precision_loss)]
    let file_size_value = bytes.len() as f64;
    #[allow(clippy::cast_possible_wrap)]
    if file_size_i64 > MAX_IFC_SIZE_BYTES as i64 {
        return error_response("IFC model must be 25 MiB or smaller", 413);
    }

    let filename = file.name();
    if !filename.to_lowercase().ends_with(".ifc") {
        return error_response("IFC file must have a .ifc extension", 400);
    }

    let version_id = uuid::Uuid::new_v4().to_string();
    let key = format!("ifcs/{version_id}/{filename}");
    let http_metadata = HttpMetadata {
        content_type: Some("application/ifc".to_string()),
        ..Default::default()
    };

    ifcs_bucket(&ctx)?
        .put(&key, bytes)
        .http_metadata(http_metadata)
        .execute()
        .await?;

    let created_at = now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| worker::Error::RustError(format!("failed to format timestamp: {e}")))?;

    db(&ctx)?
        .prepare(
            "INSERT INTO project_versions (id, project_id, filename, ifc_key, state, created_at, file_size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(&[
            version_id.clone().into(),
            project_id.clone().into(),
            filename.into(),
            key.clone().into(),
            VersionState::Undefined.as_str().into(),
            created_at.into(),
            file_size_value.into(),
        ])?
        .run()
        .await?;

    // Clear any cached GLB for this project so the new version can be reconverted.
    let glb_cache_key = glb_key_for_project(&project_id);
    let _ = ifcs_bucket(&ctx)?.delete(&glb_cache_key).await;

    Response::from_json(
        &serde_json::json!({ "ifc_key": key, "version_id": version_id, "content_type": "application/ifc", "file_size": file_size_i64 }),
    )
}

/// Deletes all IFC versions for a project from R2 and the `project_versions`
/// table. Restricted to project editors.
pub async fn delete_project_ifc(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "ifc_delete").await? {
        return Ok(response);
    }
    let account = require_account(&req, &ctx).await?;
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return error_response("Forbidden", 403);
    }

    let versions = fetch_project_versions(&ctx, &project_id, true).await?;
    for version in versions {
        let _ = ifcs_bucket(&ctx)?.delete(&version.ifc_key).await;
    }

    let glb_cache_key = glb_key_for_project(&project_id);
    let _ = ifcs_bucket(&ctx)?.delete(&glb_cache_key).await;

    db(&ctx)?
        .prepare("DELETE FROM project_versions WHERE project_id = ?1")
        .bind(&[project_id.into()])?
        .run()
        .await?;

    Response::from_json(&serde_json::json!({ "deleted": true }))
}

/// Responds with the IFC versions for a project. Undefined versions are omitted
/// unless the caller can edit the project.
pub async fn list_project_versions(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;

    let include_undefined = match require_account(&req, &ctx).await {
        Ok(account) => can_edit_project(&account, &project),
        Err(_) => false,
    };

    let versions = fetch_project_versions(&ctx, &project_id, include_undefined).await?;
    Response::from_json(&versions)
}

/// Updates the state of a single project version. Restricted to project editors.
pub async fn update_project_version(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let version_id = ctx.param("version_id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return error_response("Forbidden", 403);
    }

    let patch: VersionPatch = req.json().await?;
    let state = VersionState::parse(&patch.state)?;

    db(&ctx)?
        .prepare("UPDATE project_versions SET state = ?1 WHERE id = ?2 AND project_id = ?3")
        .bind(&[
            state.as_str().into(),
            version_id.clone().into(),
            project_id.into(),
        ])?
        .run()
        .await?;

    // Clear the GLB cache so the visibility change can be reflected on next fetch.
    let glb_cache_key = glb_key_for_project(&project.id);
    let _ = ifcs_bucket(&ctx)?.delete(&glb_cache_key).await;

    Response::empty()
}

/// Deletes a single project version. Restricted to project editors.
pub async fn delete_project_version(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let version_id = ctx.param("version_id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return error_response("Forbidden", 403);
    }

    let result = db(&ctx)?
        .prepare("SELECT ifc_key FROM project_versions WHERE id = ?1 AND project_id = ?2")
        .bind(&[version_id.clone().into(), project_id.clone().into()])?
        .all()
        .await?;
    let keys: Vec<VersionKey> = result.results::<VersionKey>()?;

    db(&ctx)?
        .prepare("DELETE FROM project_versions WHERE id = ?1 AND project_id = ?2")
        .bind(&[version_id.into(), project_id.clone().into()])?
        .run()
        .await?;

    for key in keys {
        let _ = ifcs_bucket(&ctx)?.delete(&key.ifc_key).await;
    }

    let glb_cache_key = glb_key_for_project(&project_id);
    let _ = ifcs_bucket(&ctx)?.delete(&glb_cache_key).await;

    Response::empty()
}

/// Serves an IFC model from R2 by its version id and filename.
pub async fn serve_ifc(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let version_id = ctx.param("version_id").cloned().unwrap_or_default();
    let filename = ctx.param("filename").cloned().unwrap_or_default();
    if version_id.is_empty() || filename.is_empty() {
        return error_response("Invalid IFC key", 400);
    }

    let result = db(&ctx)?
        .prepare("SELECT ifc_key FROM project_versions WHERE id = ?1 AND filename = ?2")
        .bind(&[version_id.into(), filename.clone().into()])?
        .all()
        .await?;
    let keys: Vec<VersionKey> = result.results::<VersionKey>()?;
    let Some(key) = keys.into_iter().next() else {
        return error_response("Not found", 404);
    };

    let object = ifcs_bucket(&ctx)?.get(&key.ifc_key).execute().await?;

    let Some(object) = object else {
        return error_response("Not found", 404);
    };

    let http_metadata = object.http_metadata();
    let content_type = http_metadata
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let body = object
        .body()
        .ok_or_else(|| worker::Error::RustError("IFC object has no body".into()))?;

    let headers = Headers::new();
    headers.set("Content-Type", &content_type)?;
    headers.set(
        "Content-Disposition",
        &format!("attachment; filename=\"{filename}\""),
    )?;
    Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers))
}

/// Returns the R2 key used to cache a project's converted GLB.
fn glb_key_for_project(id: &str) -> String {
    format!("ifcs/{id}/model.glb")
}

/// Serves a project's IFC model converted to a binary GLB for the 3D viewer.
pub async fn serve_project_glb(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    if id.is_empty() {
        return error_response("Invalid project id", 400);
    }

    let Some(key) = latest_visible_ifc_key(&ctx, &id).await? else {
        return error_response("No IFC model uploaded for this project", 404);
    };

    let glb_cache_key = glb_key_for_project(&id);
    if let Some(cached) = ifcs_bucket(&ctx)?.get(&glb_cache_key).execute().await? {
        let body = cached
            .body()
            .ok_or_else(|| worker::Error::RustError("GLB cache object has no body".into()))?;
        let http_metadata = cached.http_metadata();
        let content_type = http_metadata
            .content_type
            .unwrap_or_else(|| "model/gltf-binary".to_string());
        let headers = Headers::new();
        headers.set("Content-Type", &content_type)?;
        headers.set("Cache-Control", "public, max-age=3600")?;
        return Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers));
    }

    match ensure_glb_cached(&ctx, &id, &key).await? {
        Some(GlbConversion::Success(bytes)) => {
            let headers = Headers::new();
            headers.set("Content-Type", "model/gltf-binary")?;
            headers.set("Cache-Control", "public, max-age=3600")?;
            Response::from_bytes(bytes).map(|resp| resp.with_headers(headers))
        }
        Some(GlbConversion::NoGeometry) => {
            error_response("IFC model has no renderable geometry", 422)
        }
        // The cache was populated between the check above and this call; serve it.
        None => {
            let cached = ifcs_bucket(&ctx)?
                .get(glb_key_for_project(&id))
                .execute()
                .await?
                .ok_or_else(|| worker::Error::RustError("GLB cache object has no body".into()))?;
            let body = cached
                .body()
                .ok_or_else(|| worker::Error::RustError("GLB cache object has no body".into()))?;
            let http_metadata = cached.http_metadata();
            let content_type = http_metadata
                .content_type
                .unwrap_or_else(|| "model/gltf-binary".to_string());
            let headers = Headers::new();
            headers.set("Content-Type", &content_type)?;
            headers.set("Cache-Control", "public, max-age=3600")?;
            Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers))
        }
    }
}

/// Eagerly converts a project's IFC model to GLB and caches it, returning the
/// pipeline status so the client can report progress without waiting for the
/// first viewer open.
///
/// The caller must be able to edit the project. Returns a JSON body describing
/// whether the conversion succeeded, failed, or found no renderable geometry.
pub async fn convert_project_glb(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "glb_convert").await? {
        return Ok(response);
    }
    let account = require_account(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return error_response("Forbidden", 403);
    }

    let Some(key) = latest_ifc_key(&ctx, &id).await? else {
        return error_response("No IFC model uploaded for this project", 404);
    };

    match ensure_glb_cached(&ctx, &id, &key).await? {
        Some(GlbConversion::Success(_)) | None => {
            Response::from_json(&serde_json::json!({ "status": "ready" }))
        }
        Some(GlbConversion::NoGeometry) => {
            error_response("IFC model has no renderable geometry", 422)
        }
    }
}

/// Outcome of converting a project's IFC model to GLB.
enum GlbConversion {
    /// The conversion produced valid geometry.
    Success(Vec<u8>),
    /// The IFC model contained no renderable geometry.
    NoGeometry,
}

/// Converts the IFC object at `key` to GLB, caches it under the project's GLB
/// key, and returns the outcome. Cache hits return `None` so callers can serve
/// the cached body directly.
async fn ensure_glb_cached(
    ctx: &RouteContext<()>,
    id: &str,
    ifc_key: &str,
) -> Result<Option<GlbConversion>> {
    let glb_cache_key = glb_key_for_project(id);
    if ifcs_bucket(ctx)?
        .get(&glb_cache_key)
        .execute()
        .await?
        .is_some()
    {
        return Ok(None);
    }

    let object = ifcs_bucket(ctx)?
        .get(ifc_key)
        .execute()
        .await?
        .ok_or_else(|| worker::Error::RustError("IFC model not found".into()))?;
    let body = object
        .body()
        .ok_or_else(|| worker::Error::RustError("IFC object has no body".into()))?;
    let ifc_bytes = body.bytes().await?;

    let glb_bytes = export_glb(
        &ifc_bytes,
        &GltfOptions {
            include_metadata: true,
            ..GltfOptions::default()
        },
    );
    if glb_bytes.len() <= 12 {
        return Ok(Some(GlbConversion::NoGeometry));
    }

    let cache_metadata = HttpMetadata {
        content_type: Some("model/gltf-binary".to_string()),
        ..Default::default()
    };
    ifcs_bucket(ctx)?
        .put(&glb_cache_key, glb_bytes.clone())
        .http_metadata(cache_metadata)
        .execute()
        .await?;

    Ok(Some(GlbConversion::Success(glb_bytes)))
}

/// Whether the given account may edit or delete the project.
fn can_edit_project(account: &Account, project: &Project) -> bool {
    account.role == "admin"
        || account.id == project.author_id
        || project.collaborator_ids.contains(&account.id)
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

/// Increments the download counter for a project and returns the updated project.
///
/// This endpoint is rate-limited per client IP to discourage abuse while still
/// allowing legitimate downloads.
pub async fn increment_project_downloads(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    console_log!(
        "increment_project_downloads: received request for id {:?}",
        ctx.param("id")
    );

    if let Some(rate_limited) = check_rate_limit(&req, &ctx, "downloads").await? {
        console_log!("increment_project_downloads: rate limit exceeded");
        return Ok(rate_limited);
    }

    let id = ctx.param("id").cloned().unwrap_or_default();
    console_log!("increment_project_downloads: looking up project {id}");

    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;

    console_log!("increment_project_downloads: updating download count for {id}");
    db(&ctx)?
        .prepare("UPDATE projects SET downloads = downloads + 1 WHERE id = ?1")
        .bind(&[project.id.clone().into()])?
        .run()
        .await?;

    let updated = fetch_project(&ctx, &project.id)
        .await?
        .ok_or_else(|| worker::Error::RustError("updated project not found".into()))?;
    console_log!("increment_project_downloads: returning updated project {id}");
    Response::from_json(&updated)
}

/// Fetches all versions for a project, optionally including undefined-state
/// versions. Versions are ordered newest first.
async fn fetch_project_versions(
    ctx: &RouteContext<()>,
    project_id: &str,
    include_undefined: bool,
) -> Result<Vec<ProjectVersion>> {
    let sql = if include_undefined {
        "SELECT id, project_id, filename, ifc_key, state, created_at, file_size FROM project_versions WHERE project_id = ?1 ORDER BY created_at DESC"
    } else {
        "SELECT id, project_id, filename, ifc_key, state, created_at, file_size FROM project_versions WHERE project_id = ?1 AND state != 'undefined' ORDER BY created_at DESC"
    };
    let result = db(ctx)?
        .prepare(sql)
        .bind(&[project_id.into()])?
        .all()
        .await?;
    result.results::<ProjectVersion>()
}

/// Returns the R2 key of the most recent IFC version for a project regardless
/// of state. Used by editors to convert a newly uploaded (undefined) version.
async fn latest_ifc_key(ctx: &RouteContext<()>, project_id: &str) -> Result<Option<String>> {
    let versions = fetch_project_versions(ctx, project_id, true).await?;
    let key = versions.into_iter().next().map(|version| version.ifc_key);
    Ok(key)
}

/// Returns the R2 key of the best visible IFC version for a project.
///
/// Prefers the newest version with the most mature state among visible
/// versions. Returns `None` when no visible version exists.
async fn latest_visible_ifc_key(
    ctx: &RouteContext<()>,
    project_id: &str,
) -> Result<Option<String>> {
    let versions = fetch_project_versions(ctx, project_id, false).await?;
    let key = versions.into_iter().next().map(|version| version.ifc_key);
    Ok(key)
}

/// Fetches a single project by id, returning `None` when no row matches.
async fn fetch_project(ctx: &RouteContext<()>, id: &str) -> Result<Option<Project>> {
    let result = db(ctx)?
        .prepare(format!("{SELECT_PROJECT_COLUMNS} WHERE id = ?1"))
        .bind(&[id.into()])?
        .all()
        .await?;
    let projects: Vec<Project> = result.results::<Project>()?;
    Ok(projects.into_iter().next())
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
            bio: String::new(),
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
            description: String::new(),
            tags: vec![],
            platforms: vec![],
            downloads: 0,
            favorites: vec![],
            timestamp: "2025-01-01T00:00:00Z".into(),
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
            description: String::new(),
            tags: vec![],
            platforms: vec![],
            downloads: 0,
            favorites: vec![],
            timestamp: "2025-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn payload_with_valid_title_passes() {
        assert!(validate_project_payload(&sample_payload()).is_empty());
    }

    #[test]
    fn payload_with_long_title_fails() {
        let mut payload = sample_payload();
        payload.title = "a".repeat(101);
        let errors = validate_project_payload(&payload);
        assert_eq!(
            errors.get("title"),
            Some(&"Title must be 100 characters or fewer".to_string())
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
