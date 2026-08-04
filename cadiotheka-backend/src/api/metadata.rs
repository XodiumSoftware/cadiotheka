use serde::{Deserialize, Serialize};
use worker::{D1Database, Request, Response, Result, RouteContext};

use crate::DB_BINDING;

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

/// Returns the D1 database binding configured for this worker.
fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1(DB_BINDING)
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
