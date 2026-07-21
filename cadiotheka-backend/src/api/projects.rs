use serde::{Deserialize, Serialize};
use worker::*;

use crate::DB_BINDING;
use crate::api::accounts::Account;
use crate::api::session::require_account;
use crate::utils::js_option;

const SELECT_PROJECT_COLUMNS: &str = "SELECT id, title, author, author_id, author_username, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url FROM projects";

/// Maximum allowed length for a project title.
const MAX_TITLE_LENGTH: usize = 100;
/// Maximum allowed length for a project short description.
const MAX_DESCRIPTION_LENGTH: usize = 500;

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
    pub description: String,
    pub extended_desc: String,
    #[serde(with = "json_string")]
    pub tags: Vec<String>,
    #[serde(with = "json_string")]
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
    pub author_username: String,
    pub description: String,
    pub extended_desc: String,
    #[serde(with = "json_string")]
    pub tags: Vec<String>,
    #[serde(with = "json_string")]
    pub supported_platforms: Vec<String>,
    pub downloads: u64,
    pub favorites: u64,
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

    db(&ctx)?
        .prepare(
            "INSERT INTO projects (id, title, author, author_id, author_username, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .bind(&[
            payload.id.into(),
            payload.title.into(),
            payload.author.into(),
            payload.author_id.into(),
            payload.author_username.into(),
            payload.description.into(),
            payload.extended_desc.into(),
            tags.into(),
            platforms.into(),
            (payload.downloads as f64).into(),
            (payload.favorites as f64).into(),
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

    db(&ctx)?
        .prepare(
            "UPDATE projects \
             SET title = ?1, author = ?2, author_id = ?3, author_username = ?4, description = ?5, extended_desc = ?6, tags = ?7, supported_platforms = ?8, downloads = ?9, favorites = ?10, timestamp = ?11, icon_url = ?12 \
             WHERE id = ?13",
        )
        .bind(&[
            payload.title.into(),
            payload.author.into(),
            payload.author_id.into(),
            payload.author_username.into(),
            payload.description.into(),
            payload.extended_desc.into(),
            tags.into(),
            platforms.into(),
            (payload.downloads as f64).into(),
            (payload.favorites as f64).into(),
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
    Response::empty()
}

/// Whether the given account may edit or delete the project.
fn can_edit_project(account: &Account, project: &Project) -> bool {
    account.role == "admin" || account.id == project.author_id
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
            provider: "seed".into(),
            provider_id: "seed_acc-1".into(),
        }
    }

    fn sample_project(author_id: &str) -> Project {
        Project {
            id: "proj-1".into(),
            title: "Sample".into(),
            author: "Author".into(),
            author_id: author_id.into(),
            author_username: "author".into(),
            description: "".into(),
            extended_desc: "".into(),
            tags: vec![],
            supported_platforms: vec![],
            downloads: 0,
            favorites: 0,
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
            description: "A short description.".into(),
            extended_desc: "".into(),
            tags: vec![],
            supported_platforms: vec![],
            downloads: 0,
            favorites: 0,
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
