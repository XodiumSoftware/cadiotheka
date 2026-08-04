use serde::{Deserialize, Serialize};
use worker::{D1Database, Request, Response, Result, RouteContext};

use crate::DB_BINDING;
use crate::api::accounts::Account;
use crate::api::session::require_account;
use crate::utils::{error_response, rust_err};

/// A content tag stored in D1.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagRecord {
    pub id: String,
    pub label: String,
    pub color: String,
}

/// A supported CAD platform stored in D1.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlatformRecord {
    pub id: String,
    pub label: String,
    pub color: String,
}

/// Payload used to create a new tag.
#[derive(Deserialize, Debug)]
pub struct TagPayload {
    pub id: String,
    pub label: String,
    pub color: String,
}

/// Payload used to update an existing tag.
#[derive(Deserialize, Debug)]
pub struct TagUpdatePayload {
    pub label: String,
    pub color: String,
}

/// Payload used to create a new platform.
#[derive(Deserialize, Debug)]
pub struct PlatformPayload {
    pub id: String,
    pub label: String,
    pub color: String,
}

/// Payload used to update an existing platform.
#[derive(Deserialize, Debug)]
pub struct PlatformUpdatePayload {
    pub label: String,
    pub color: String,
}

/// Returns the D1 database binding configured for this worker.
fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1(DB_BINDING)
}

/// Requires the request to come from an authenticated admin account.
async fn require_admin(req: &Request, ctx: &RouteContext<()>) -> Result<Account> {
    let account = require_account(req, ctx).await?;
    if account.role != "admin" {
        return Err(worker::Error::RustError("Forbidden".into()));
    }
    Ok(account)
}

/// Normalizes a metadata id into a safe wire identifier.
///
/// Returns an error if the id is empty or contains characters other than
/// lowercase ASCII letters, digits, and underscores.
fn normalize_metadata_id(id: &str) -> Result<String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(rust_err("id is required"));
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        Ok(trimmed.to_string())
    } else {
        Err(rust_err(
            "id must contain only lowercase letters, digits, and underscores",
        ))
    }
}

/// Returns `Ok(true)` if a tag with the given id exists.
async fn tag_exists(ctx: &RouteContext<()>, id: &str) -> Result<bool> {
    #[derive(Deserialize)]
    struct Row {
        #[allow(dead_code)]
        id: String,
    }
    let result = db(ctx)?
        .prepare("SELECT id FROM tags WHERE id = ?1")
        .bind(&[id.into()])?
        .all()
        .await?;
    let rows: Vec<Row> = result.results::<Row>()?;
    Ok(!rows.is_empty())
}

/// Returns `Ok(true)` if a platform with the given id exists.
async fn platform_exists(ctx: &RouteContext<()>, id: &str) -> Result<bool> {
    #[derive(Deserialize)]
    struct Row {
        #[allow(dead_code)]
        id: String,
    }
    let result = db(ctx)?
        .prepare("SELECT id FROM platforms WHERE id = ?1")
        .bind(&[id.into()])?
        .all()
        .await?;
    let rows: Vec<Row> = result.results::<Row>()?;
    Ok(!rows.is_empty())
}

/// Responds with a JSON array of all content tags.
pub async fn list_tags(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let result = db(&ctx)?
        .prepare("SELECT id, label, color FROM tags ORDER BY label")
        .all()
        .await?;
    let tags: Vec<TagRecord> = result.results::<TagRecord>()?;
    Response::from_json(&tags)
}

/// Responds with a JSON array of all supported CAD platforms.
pub async fn list_platforms(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let result = db(&ctx)?
        .prepare("SELECT id, label, color FROM platforms ORDER BY label")
        .all()
        .await?;
    let platforms: Vec<PlatformRecord> = result.results::<PlatformRecord>()?;
    Response::from_json(&platforms)
}

/// Creates a new content tag. Restricted to admins.
pub async fn create_tag(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin(&req, &ctx).await?;
    let payload: TagPayload = req.json().await?;
    let id = normalize_metadata_id(&payload.id)?;

    if payload.label.trim().is_empty() {
        return error_response("label is required", 400);
    }
    if payload.color.trim().is_empty() {
        return error_response("color is required", 400);
    }
    if tag_exists(&ctx, &id).await? {
        return error_response("tag id already exists", 409);
    }

    db(&ctx)?
        .prepare("INSERT INTO tags (id, label, color) VALUES (?1, ?2, ?3)")
        .bind(&[
            id.clone().into(),
            payload.label.clone().into(),
            payload.color.clone().into(),
        ])?
        .run()
        .await?;

    let record = TagRecord {
        id,
        label: payload.label,
        color: payload.color,
    };
    Response::from_json(&record)
}

/// Updates the label and color of an existing content tag. Restricted to admins.
pub async fn update_tag(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let payload: TagUpdatePayload = req.json().await?;

    if payload.label.trim().is_empty() {
        return error_response("label is required", 400);
    }
    if payload.color.trim().is_empty() {
        return error_response("color is required", 400);
    }
    if !tag_exists(&ctx, &id).await? {
        return error_response("tag not found", 404);
    }

    db(&ctx)?
        .prepare("UPDATE tags SET label = ?1, color = ?2 WHERE id = ?3")
        .bind(&[
            payload.label.clone().into(),
            payload.color.clone().into(),
            id.clone().into(),
        ])?
        .run()
        .await?;

    let record = TagRecord {
        id,
        label: payload.label,
        color: payload.color,
    };
    Response::from_json(&record)
}

/// Deletes a content tag. Restricted to admins.
pub async fn delete_tag(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    db(&ctx)?
        .prepare("DELETE FROM tags WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;
    Response::empty()
}

/// Creates a new supported CAD platform. Restricted to admins.
pub async fn create_platform(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin(&req, &ctx).await?;
    let payload: PlatformPayload = req.json().await?;
    let id = normalize_metadata_id(&payload.id)?;

    if payload.label.trim().is_empty() {
        return error_response("label is required", 400);
    }
    if payload.color.trim().is_empty() {
        return error_response("color is required", 400);
    }
    if platform_exists(&ctx, &id).await? {
        return error_response("platform id already exists", 409);
    }

    db(&ctx)?
        .prepare("INSERT INTO platforms (id, label, color) VALUES (?1, ?2, ?3)")
        .bind(&[
            id.clone().into(),
            payload.label.clone().into(),
            payload.color.clone().into(),
        ])?
        .run()
        .await?;

    let record = PlatformRecord {
        id,
        label: payload.label,
        color: payload.color,
    };
    Response::from_json(&record)
}

/// Updates the label and color of an existing platform. Restricted to admins.
pub async fn update_platform(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    let payload: PlatformUpdatePayload = req.json().await?;

    if payload.label.trim().is_empty() {
        return error_response("label is required", 400);
    }
    if payload.color.trim().is_empty() {
        return error_response("color is required", 400);
    }
    if !platform_exists(&ctx, &id).await? {
        return error_response("platform not found", 404);
    }

    db(&ctx)?
        .prepare("UPDATE platforms SET label = ?1, color = ?2 WHERE id = ?3")
        .bind(&[
            payload.label.clone().into(),
            payload.color.clone().into(),
            id.clone().into(),
        ])?
        .run()
        .await?;

    let record = PlatformRecord {
        id,
        label: payload.label,
        color: payload.color,
    };
    Response::from_json(&record)
}

/// Deletes a supported CAD platform. Restricted to admins.
pub async fn delete_platform(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin(&req, &ctx).await?;
    let id = ctx.param("id").cloned().unwrap_or_default();
    db(&ctx)?
        .prepare("DELETE FROM platforms WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;
    Response::empty()
}
