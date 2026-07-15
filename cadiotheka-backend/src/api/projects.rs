use serde::{Deserialize, Serialize};
use worker::*;

use crate::DB_BINDING;

const SELECT_PROJECT_COLUMNS: &str = "SELECT id, title, author, author_id, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url FROM projects";

/// A Cadiotheka project stored in D1.
#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub description: String,
    pub extended_desc: String,
    pub tags: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub downloads: u64,
    pub favorites: u64,
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
    pub description: String,
    pub extended_desc: String,
    pub tags: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub downloads: u64,
    pub favorites: u64,
    pub timestamp: String,
    pub icon_url: Option<String>,
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

/// Creates a new project from the request body.
pub async fn create_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let payload: ProjectPayload = req.json().await?;
    let tags = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let platforms =
        serde_json::to_string(&payload.supported_platforms).unwrap_or_else(|_| "[]".to_string());

    db(&ctx)?
        .prepare(
            "INSERT INTO projects (id, title, author, author_id, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(&[
            payload.id.into(),
            payload.title.into(),
            payload.author.into(),
            payload.author_id.into(),
            payload.description.into(),
            payload.extended_desc.into(),
            tags.into(),
            platforms.into(),
            (payload.downloads as i64).into(),
            (payload.favorites as i64).into(),
            payload.timestamp.into(),
            payload.icon_url.into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Replaces an existing project, identified by the `:id` path parameter.
pub async fn update_project(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    let payload: ProjectPayload = req.json().await?;
    let tags = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let platforms =
        serde_json::to_string(&payload.supported_platforms).unwrap_or_else(|_| "[]".to_string());

    db(&ctx)?
        .prepare(
            "UPDATE projects \
             SET title = ?1, author = ?2, author_id = ?3, description = ?4, extended_desc = ?5, tags = ?6, supported_platforms = ?7, downloads = ?8, favorites = ?9, timestamp = ?10, icon_url = ?11 \
             WHERE id = ?12",
        )
        .bind(&[
            payload.title.into(),
            payload.author.into(),
            payload.author_id.into(),
            payload.description.into(),
            payload.extended_desc.into(),
            tags.into(),
            platforms.into(),
            (payload.downloads as i64).into(),
            (payload.favorites as i64).into(),
            payload.timestamp.into(),
            payload.icon_url.into(),
            id.into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Deletes the project identified by the `:id` path parameter.
pub async fn delete_project(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    db(&ctx)?
        .prepare("DELETE FROM projects WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;
    Response::empty()
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
