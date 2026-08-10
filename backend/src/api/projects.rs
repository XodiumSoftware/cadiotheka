use serde::{Deserialize, Serialize};
use worker::{
    FormEntry, Headers, HttpMetadata, Request, Response, Result, RouteContext, console_log,
};

use crate::api::accounts::{Account, Role};
use crate::api::session::require_account;
use crate::guards::{
    GuardOutcome, require_auth_with_rate_limit, require_auth_with_turnstile_and_rate_limit,
};
use crate::utils::{
    RateLimitNamespace, assets_bucket, bad_request, check_rate_limit, db, error_response,
    forbidden, not_found, now_utc,
};
use ifc_lite_export::{GltfOptions, export_glb};

const SELECT_PROJECT_COLUMNS: &str = "SELECT id, title, author, author_id, author_username, collaborator_ids, description, tags, downloads, favorites, timestamp FROM projects";

/// Maximum allowed length for a project title.
const MAX_TITLE_LENGTH: usize = 100;
/// Maximum allowed length for a project description.
const MAX_DESCRIPTION_LENGTH: usize = 5000;
/// Maximum allowed size for an uploaded project IFC model, in bytes.
const MAX_IFC_SIZE_BYTES: usize = 25 * 1024 * 1024; // 25 MiB

/// Content tags for projects, mirroring the frontend tag set so the backend can
/// validate and round-trip the same wire ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tag {
    /// Three-dimensional models and assets.
    #[serde(rename = "3d_model")]
    ThreeDModel,
    /// Two-dimensional drawings and diagrams.
    #[serde(rename = "2d_drawing")]
    TwoDDrawing,
    /// Parametric or algorithmically defined designs.
    Parametric,
    /// Designs intended for fabrication or manufacturing.
    Fabrication,
    /// Robotics parts, assemblies, and accessories.
    Robotics,
    /// Furniture designs.
    Furniture,
    /// Vehicles and vehicle parts.
    Vehicle,
    /// Architectural models and elements.
    Architecture,
    /// Electronics enclosures and components.
    Electronics,
    /// Tools, jigs, and workshop helpers.
    Tooling,
    /// Lighting fixtures and designs.
    Lighting,
    /// Do-it-yourself projects and hacks.
    Diy,
    /// Interior design objects and layouts.
    Interior,
    /// General engineering models.
    Engineering,
    /// Aerospace parts and assemblies.
    Aerospace,
    /// Decorative objects.
    Decor,
    /// Medical devices and helpers.
    Medical,
    /// Assets for games and real-time rendering.
    GameAsset,
    /// Artistic or sculptural models.
    Art,
    /// Educational models and demonstrations.
    Educational,
    /// Work-in-progress designs.
    Wip,
}

impl Tag {
    /// Stable wire id stored on project rows.
    pub fn id(self) -> &'static str {
        match self {
            Self::ThreeDModel => "3d_model",
            Self::TwoDDrawing => "2d_drawing",
            Self::Parametric => "parametric",
            Self::Fabrication => "fabrication",
            Self::Robotics => "robotics",
            Self::Furniture => "furniture",
            Self::Vehicle => "vehicle",
            Self::Architecture => "architecture",
            Self::Electronics => "electronics",
            Self::Tooling => "tooling",
            Self::Lighting => "lighting",
            Self::Diy => "diy",
            Self::Interior => "interior",
            Self::Engineering => "engineering",
            Self::Aerospace => "aerospace",
            Self::Decor => "decor",
            Self::Medical => "medical",
            Self::GameAsset => "game_asset",
            Self::Art => "art",
            Self::Educational => "educational",
            Self::Wip => "wip",
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

/// A version state for an IFC file.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionState {
    Undefined,
    Alpha,
    Beta,
    Stable,
}

impl std::fmt::Display for VersionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Undefined => "undefined",
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Stable => "stable",
        };
        write!(f, "{s}")
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
    pub version: String,
    pub downloads: i64,
}

/// Payload used to patch a project version.
#[derive(Deserialize, Debug)]
pub struct VersionPatch {
    pub state: Option<VersionState>,
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
    #[serde(with = "json_tags")]
    pub tags: Vec<Tag>,
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
    #[serde(with = "json_tags")]
    pub tags: Vec<Tag>,
    pub downloads: u64,
    #[serde(with = "json_string")]
    pub favorites: Vec<String>,
    pub timestamp: String,
}

/// Serde adapter that stores a `Vec<String>` as a single JSON string column.
///
/// D1 stores tags, favorites, and collaborators as TEXT containing a JSON
/// array, so we serialize to a JSON string on the way in and parse that JSON
/// string on the way out.
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

mod json_tags {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::Tag;

    pub fn serialize<S: Serializer>(value: &Vec<Tag>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Tag>, D::Error> {
        let s = String::deserialize(deserializer)?;
        serde_json::from_str(&s).map_err(serde::de::Error::custom)
    }
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
        None => not_found("Not found"),
    }
}

/// Creates a new project from the request body, attributing it to the
/// authenticated user.
pub async fn create_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = match require_auth_with_turnstile_and_rate_limit(
        &mut req,
        &ctx,
        RateLimitNamespace::ProjectCreate,
    )
    .await?
    {
        GuardOutcome::Account(account) => account,
        GuardOutcome::Response(resp) => return Ok(resp),
    };
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

    let tags = payload
        .tags
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    let favorites = serde_json::to_string(&payload.favorites).unwrap_or_else(|_| "[]".to_string());
    let collaborator_ids =
        serde_json::to_string(&payload.collaborator_ids).unwrap_or_else(|_| "[]".to_string());

    #[allow(clippy::cast_precision_loss)]
    let downloads_value = payload.downloads as f64;

    db(&ctx)?
        .prepare(
            "INSERT INTO projects (id, title, author, author_id, author_username, collaborator_ids, description, tags, downloads, favorites, timestamp) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
    tags: Option<Vec<Tag>>,
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
        return forbidden("Forbidden");
    }

    let patch: ProjectPatch = req.json().await?;
    if let Some(title) = patch.title {
        if title.len() > MAX_TITLE_LENGTH {
            return bad_request("Title must be 100 characters or fewer");
        }
        db(&ctx)?
            .prepare("UPDATE projects SET title = ?1 WHERE id = ?2")
            .bind(&[title.into(), id.clone().into()])?
            .run()
            .await?;
    }

    if let Some(tags) = patch.tags {
        let tags = tags
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        let tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        db(&ctx)?
            .prepare("UPDATE projects SET tags = ?1 WHERE id = ?2")
            .bind(&[tags.into(), id.clone().into()])?
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
            return bad_request("Description must be 5000 characters or fewer");
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
        return forbidden("Forbidden");
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
    let tags = payload
        .tags
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    payload.collaborator_ids = project.collaborator_ids.clone();
    let favorites = serde_json::to_string(&project.favorites).unwrap_or_else(|_| "[]".to_string());
    let collaborator_ids =
        serde_json::to_string(&payload.collaborator_ids).unwrap_or_else(|_| "[]".to_string());

    #[allow(clippy::cast_precision_loss)]
    let downloads_value = payload.downloads as f64;

    db(&ctx)?
        .prepare(
            "UPDATE projects \
             SET title = ?1, author = ?2, author_id = ?3, author_username = ?4, collaborator_ids = ?5, description = ?6, tags = ?7, downloads = ?8, favorites = ?9, timestamp = ?10 \
             WHERE id = ?11",
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
        return forbidden("Forbidden");
    }

    db(&ctx)?
        .prepare("DELETE FROM projects WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;

    Response::empty()
}

/// Handles a multipart upload of a project IFC model, stores it in R2,
/// and creates a new `project_versions` row with state `undefined`.
pub async fn upload_project_ifc(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account =
        match require_auth_with_rate_limit(&req, &ctx, RateLimitNamespace::IfcUpload).await? {
            GuardOutcome::Account(account) => account,
            GuardOutcome::Response(resp) => return Ok(resp),
        };
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return forbidden("Forbidden");
    }

    let form_data = req.form_data().await?;
    let Some(FormEntry::File(file)) = form_data.get("ifc") else {
        return bad_request("missing IFC file");
    };
    let version = match form_data.get("version") {
        Some(FormEntry::Field(value)) if !value.is_empty() => value,
        _ => "1.0.0".to_string(),
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
        return bad_request("IFC file must have a .ifc extension");
    }

    let version_id = uuid::Uuid::new_v4().to_string();
    let key = format!("ifcs/{version_id}/{filename}");
    let http_metadata = HttpMetadata {
        content_type: Some("application/ifc".to_string()),
        ..Default::default()
    };

    assets_bucket(&ctx)?
        .put(&key, bytes)
        .http_metadata(http_metadata)
        .execute()
        .await?;

    let created_at = now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| worker::Error::RustError(format!("failed to format timestamp: {e}")))?;

    db(&ctx)?
        .prepare(
            "INSERT INTO project_versions (id, project_id, filename, ifc_key, state, created_at, file_size, version, downloads) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&[
            version_id.clone().into(),
            project_id.clone().into(),
            filename.into(),
            key.clone().into(),
            VersionState::Undefined.to_string().into(),
            created_at.into(),
            file_size_value.into(),
            version.clone().into(),
            0_f64.into(),
        ])?
        .run()
        .await?;

    // Clear any cached GLB for this project so the new version can be reconverted.
    let glb_cache_key = glb_key_for_project(&project_id);
    let _ = assets_bucket(&ctx)?.delete(&glb_cache_key).await;

    Response::from_json(&serde_json::json!({
        "ifc_key": key,
        "version_id": version_id,
        "content_type": "application/ifc",
        "file_size": file_size_i64,
        "version": version,
        "downloads": 0,
    }))
}

/// Deletes all IFC versions for a project from R2 and the `project_versions`
/// table. Restricted to project editors.
pub async fn delete_project_ifc(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account =
        match require_auth_with_rate_limit(&req, &ctx, RateLimitNamespace::IfcDelete).await? {
            GuardOutcome::Account(account) => account,
            GuardOutcome::Response(resp) => return Ok(resp),
        };
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return forbidden("Forbidden");
    }

    let versions = fetch_project_versions(&ctx, &project_id, true).await?;
    for version in versions {
        let _ = assets_bucket(&ctx)?.delete(&version.ifc_key).await;
    }

    let glb_cache_key = glb_key_for_project(&project_id);
    let _ = assets_bucket(&ctx)?.delete(&glb_cache_key).await;

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

/// Patches a single project version (state only). Restricted to project editors.
pub async fn update_project_version(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let project_id = ctx.param("id").cloned().unwrap_or_default();
    let version_id = ctx.param("version_id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &project_id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return forbidden("Forbidden");
    }

    let patch: VersionPatch = req.json().await?;

    if let Some(state) = patch.state {
        db(&ctx)?
            .prepare("UPDATE project_versions SET state = ?1 WHERE id = ?2 AND project_id = ?3")
            .bind(&[
                state.to_string().into(),
                version_id.clone().into(),
                project_id.clone().into(),
            ])?
            .run()
            .await?;

        let glb_cache_key = glb_key_for_project(&project.id);
        let _ = assets_bucket(&ctx)?.delete(&glb_cache_key).await;
    }

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
        return forbidden("Forbidden");
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
        let _ = assets_bucket(&ctx)?.delete(&key.ifc_key).await;
    }

    let glb_cache_key = glb_key_for_project(&project_id);
    let _ = assets_bucket(&ctx)?.delete(&glb_cache_key).await;

    Response::empty()
}

/// Serves an IFC model from R2 by its version id and filename.
pub async fn serve_ifc(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let version_id = ctx.param("version_id").cloned().unwrap_or_default();
    let filename = ctx.param("filename").cloned().unwrap_or_default();
    if version_id.is_empty() || filename.is_empty() {
        return bad_request("Invalid IFC key");
    }

    let result = db(&ctx)?
        .prepare("SELECT ifc_key FROM project_versions WHERE id = ?1 AND filename = ?2")
        .bind(&[version_id.clone().into(), filename.clone().into()])?
        .all()
        .await?;
    let keys: Vec<VersionKey> = result.results::<VersionKey>()?;
    let Some(key) = keys.into_iter().next() else {
        return not_found("Not found");
    };

    let object = assets_bucket(&ctx)?.get(&key.ifc_key).execute().await?;

    let Some(object) = object else {
        return not_found("Not found");
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

    // Increment the per-version download counter.
    let _ = db(&ctx)?
        .prepare("UPDATE project_versions SET downloads = downloads + 1 WHERE id = ?1")
        .bind(&[version_id.into()])?
        .run()
        .await;

    Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers))
}

/// Returns the R2 key used to cache a project's converted GLB.
fn glb_key_for_project(id: &str) -> String {
    format!("ifcs/{id}/model.glb")
}

/// Returns the R2 key used to cache per-primitive metadata for the GLB.
fn glb_metadata_key_for_project(id: &str) -> String {
    format!("ifcs/{id}/model-metadata.json")
}

/// Metadata for a single primitive in the converted GLB.
#[derive(Debug, Serialize, Deserialize)]
struct GltfPrimitiveMetadata {
    express_id: Option<u32>,
    name: Option<String>,
}

/// Extracts per-primitive metadata from a binary GLB.
///
/// The exporter stores the IFC express id in each node's `extras.expressId`.
/// This function walks the default scene in the same depth-first order that
/// `three-d-asset` uses when flattening nodes to primitives, so the returned
/// array index matches the primitive index used by the viewer's raycaster.
fn extract_glb_metadata(glb_bytes: &[u8]) -> Vec<GltfPrimitiveMetadata> {
    if glb_bytes.len() <= 12 {
        return Vec::new();
    }

    let json_chunk_len = match glb_bytes.get(12..16).and_then(|s| s.try_into().ok()) {
        Some(bytes) => u32::from_le_bytes(bytes) as usize,
        None => return Vec::new(),
    };
    let json_chunk_type = match glb_bytes.get(16..20).and_then(|s| s.try_into().ok()) {
        Some(bytes) => u32::from_le_bytes(bytes),
        None => return Vec::new(),
    };
    if json_chunk_type != 0x4E4F_534A {
        return Vec::new();
    }

    let json_start: usize = 20;
    let json_end = json_start.saturating_add(json_chunk_len);
    if json_end > glb_bytes.len() {
        return Vec::new();
    }
    let json: serde_json::Value = match serde_json::from_slice(&glb_bytes[json_start..json_end]) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let nodes = json
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .cloned();
    let meshes = json
        .get("meshes")
        .and_then(serde_json::Value::as_array)
        .cloned();
    let Some(nodes) = nodes else {
        return Vec::new();
    };

    let default_scene = json
        .get("scene")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        .try_into()
        .unwrap_or(0usize);
    let scenes = json
        .get("scenes")
        .and_then(serde_json::Value::as_array)
        .cloned();
    let root_indices: Vec<usize> = scenes
        .as_ref()
        .and_then(|s| s.get(default_scene))
        .and_then(|s| s.get("nodes"))
        .and_then(serde_json::Value::as_array)
        .map_or_else(
            || (0..nodes.len()).collect(),
            |arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().and_then(|n| n.try_into().ok()))
                    .collect()
            },
        );

    let mut result = Vec::new();
    for index in root_indices {
        visit_gltf_node(index, &nodes, meshes.as_deref(), &mut result);
    }
    result
}

fn visit_gltf_node(
    index: usize,
    nodes: &[serde_json::Value],
    meshes: Option<&[serde_json::Value]>,
    out: &mut Vec<GltfPrimitiveMetadata>,
) {
    let Some(node) = nodes.get(index) else {
        return;
    };

    let name = node
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(String::from);
    let express_id = node
        .get("extras")
        .and_then(|e| e.get("expressId"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|n| n.try_into().ok());

    if let Some(mesh_index) = node
        .get("mesh")
        .and_then(serde_json::Value::as_u64)
        .and_then(|n| n.try_into().ok())
    {
        let mesh_index: usize = mesh_index;
        let primitive_count = meshes
            .and_then(|list: &[serde_json::Value]| list.get(mesh_index))
            .and_then(|m: &serde_json::Value| m.get("primitives"))
            .and_then(serde_json::Value::as_array)
            .map_or(0, std::vec::Vec::len);
        for _ in 0..primitive_count {
            out.push(GltfPrimitiveMetadata {
                express_id,
                name: name.clone(),
            });
        }
    }

    let children = node
        .get("children")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for child in children {
        if let Some(child_index) = child.as_u64().and_then(|n| n.try_into().ok()) {
            visit_gltf_node(child_index, nodes, meshes, out);
        }
    }
}

/// Serves a project's IFC model converted to a binary GLB for the 3D viewer.
pub async fn serve_project_glb(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    if id.is_empty() {
        return bad_request("Invalid project id");
    }

    let Some(key) = latest_visible_ifc_key(&ctx, &id).await? else {
        return not_found("No IFC model uploaded for this project");
    };

    let glb_cache_key = glb_key_for_project(&id);
    if let Some(cached) = assets_bucket(&ctx)?.get(&glb_cache_key).execute().await? {
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

    match ensure_glb_assets_cached(&ctx, &id, &key).await? {
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
            let cached = assets_bucket(&ctx)?
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

/// Serves per-primitive metadata for a project's converted GLB.
pub async fn serve_project_glb_metadata(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    if id.is_empty() {
        return bad_request("Invalid project id");
    }

    let Some(key) = latest_visible_ifc_key(&ctx, &id).await? else {
        return not_found("No IFC model uploaded for this project");
    };

    let metadata_cache_key = glb_metadata_key_for_project(&id);
    if let Some(cached) = assets_bucket(&ctx)?
        .get(&metadata_cache_key)
        .execute()
        .await?
    {
        let body = cached.body().ok_or_else(|| {
            worker::Error::RustError("GLB metadata cache object has no body".into())
        })?;
        let http_metadata = cached.http_metadata();
        let content_type = http_metadata
            .content_type
            .unwrap_or_else(|| "application/json".to_string());
        let headers = Headers::new();
        headers.set("Content-Type", &content_type)?;
        headers.set("Cache-Control", "public, max-age=3600")?;
        return Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers));
    }

    match ensure_glb_assets_cached(&ctx, &id, &key).await? {
        Some(GlbConversion::Success(_)) | None => {
            let cached = assets_bucket(&ctx)?
                .get(glb_metadata_key_for_project(&id))
                .execute()
                .await?
                .ok_or_else(|| {
                    worker::Error::RustError("GLB metadata cache object has no body".into())
                })?;
            let body = cached.body().ok_or_else(|| {
                worker::Error::RustError("GLB metadata cache object has no body".into())
            })?;
            let http_metadata = cached.http_metadata();
            let content_type = http_metadata
                .content_type
                .unwrap_or_else(|| "application/json".to_string());
            let headers = Headers::new();
            headers.set("Content-Type", &content_type)?;
            headers.set("Cache-Control", "public, max-age=3600")?;
            Response::from_body(body.response_body()?).map(|resp| resp.with_headers(headers))
        }
        Some(GlbConversion::NoGeometry) => {
            error_response("IFC model has no renderable geometry", 422)
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
    let account =
        match require_auth_with_rate_limit(&req, &ctx, RateLimitNamespace::GlbConvert).await? {
            GuardOutcome::Account(account) => account,
            GuardOutcome::Response(resp) => return Ok(resp),
        };
    let id = ctx.param("id").cloned().unwrap_or_default();
    let project = fetch_project(&ctx, &id)
        .await?
        .ok_or_else(|| worker::Error::RustError("project not found".into()))?;
    if !can_edit_project(&account, &project) {
        return forbidden("Forbidden");
    }

    let Some(key) = latest_ifc_key(&ctx, &id).await? else {
        return not_found("No IFC model uploaded for this project");
    };

    match ensure_glb_assets_cached(&ctx, &id, &key).await? {
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

/// Converts the IFC object at `key` to GLB, caches both the GLB and a
/// per-primitive metadata file, and returns the outcome. Cache hits return
/// `None` so callers can serve the cached body directly.
async fn ensure_glb_assets_cached(
    ctx: &RouteContext<()>,
    id: &str,
    ifc_key: &str,
) -> Result<Option<GlbConversion>> {
    let glb_cache_key = glb_key_for_project(id);
    let metadata_cache_key = glb_metadata_key_for_project(id);
    let glb_cached = assets_bucket(ctx)?
        .get(&glb_cache_key)
        .execute()
        .await?
        .is_some();
    let metadata_cached = assets_bucket(ctx)?
        .get(&metadata_cache_key)
        .execute()
        .await?
        .is_some();

    if glb_cached && metadata_cached {
        return Ok(None);
    }

    let object = assets_bucket(ctx)?
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

    let metadata = extract_glb_metadata(&glb_bytes);
    let metadata_json = serde_json::to_vec(&serde_json::json!({ "primitives": metadata }))?;

    let glb_http_metadata = HttpMetadata {
        content_type: Some("model/gltf-binary".to_string()),
        ..Default::default()
    };
    let metadata_http_metadata = HttpMetadata {
        content_type: Some("application/json".to_string()),
        ..Default::default()
    };

    assets_bucket(ctx)?
        .put(&glb_cache_key, glb_bytes.clone())
        .http_metadata(glb_http_metadata)
        .execute()
        .await?;
    assets_bucket(ctx)?
        .put(&metadata_cache_key, metadata_json)
        .http_metadata(metadata_http_metadata)
        .execute()
        .await?;

    Ok(Some(GlbConversion::Success(glb_bytes)))
}

/// Whether the given account may edit or delete the project.
fn can_edit_project(account: &Account, project: &Project) -> bool {
    account.role == Role::Admin
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

    if let Some(rate_limited) = check_rate_limit(&req, &ctx, RateLimitNamespace::Downloads).await? {
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
        "SELECT id, project_id, filename, ifc_key, state, created_at, file_size, version, downloads FROM project_versions WHERE project_id = ?1 ORDER BY created_at DESC"
    } else {
        "SELECT id, project_id, filename, ifc_key, state, created_at, file_size, version, downloads FROM project_versions WHERE project_id = ?1 AND state != ?2 ORDER BY created_at DESC"
    };
    let result = db(ctx)?
        .prepare(sql)
        .bind(&[
            project_id.into(),
            VersionState::Undefined.to_string().into(),
        ])?
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

    fn sample_account(role: Role) -> Account {
        Account {
            id: "acc-1".into(),
            username: "creator".into(),
            display_name: "Creator".into(),
            email: "creator@example.com".into(),
            role,
            bio: String::new(),
            avatar_url: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            verified: 1,
            viewer_preferences: "{}".into(),
            provider: crate::api::accounts::Provider::GitHub,
            provider_id: String::new(),
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
        let account = sample_account(Role::Creator);
        let project = sample_project(&account.id);
        assert!(can_edit_project(&account, &project));
    }

    #[test]
    fn non_owner_cannot_edit_project() {
        let account = sample_account(Role::Creator);
        let project = sample_project("other");
        assert!(!can_edit_project(&account, &project));
    }

    #[test]
    fn admin_can_edit_any_project() {
        let account = sample_account(Role::Admin);
        let project = sample_project("other");
        assert!(can_edit_project(&account, &project));
    }

    #[test]
    fn extract_glb_metadata_maps_primitives_to_express_ids() {
        let glb = build_test_glb(
            r#"{
                "scene": 0,
                "scenes": [{"nodes": [0, 1]}],
                "nodes": [
                    {"name": "Wall-1", "mesh": 0, "extras": {"expressId": 123}},
                    {"name": "Door-1", "mesh": 1, "extras": {"expressId": 456}}
                ],
                "meshes": [
                    {"primitives": [{"attributes": {"POSITION": 0}}]},
                    {"primitives": [
                        {"attributes": {"POSITION": 1}},
                        {"attributes": {"POSITION": 2}}
                    ]}
                ]
            }"#,
        );

        let metadata = extract_glb_metadata(&glb);

        assert_eq!(metadata.len(), 3);
        assert_eq!(metadata[0].express_id, Some(123));
        assert_eq!(metadata[0].name.as_deref(), Some("Wall-1"));
        assert_eq!(metadata[1].express_id, Some(456));
        assert_eq!(metadata[1].name.as_deref(), Some("Door-1"));
        assert_eq!(metadata[2].express_id, Some(456));
        assert_eq!(metadata[2].name.as_deref(), Some("Door-1"));
    }

    #[test]
    fn extract_glb_metadata_returns_empty_for_short_bytes() {
        assert!(extract_glb_metadata(b"glTF\x02\x00\x00\x00").is_empty());
    }

    fn build_test_glb(json: &str) -> Vec<u8> {
        #![allow(clippy::cast_possible_truncation)]

        let json_bytes = json.as_bytes();
        let chunk_len = json_bytes.len() as u32;
        let total_len = 12 + 8 + chunk_len;
        let mut glb = Vec::with_capacity(total_len as usize);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&total_len.to_le_bytes());
        glb.extend_from_slice(&chunk_len.to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534A_u32.to_le_bytes());
        glb.extend_from_slice(json_bytes);
        glb
    }
}
