use crate::data::error::RequestError;
use crate::data::project_types::{
    ProjectCreationResult, ProjectData, ProjectVersion, ValidationErrorResponse,
};
use crate::utils::{
    project_downloads_url, project_favorites_url, project_glb_url, project_ifc_url, project_url,
    project_version_url, project_versions_url, projects_url,
};
use gloo_net::http::Request;
use serde::Deserialize;
use web_sys::RequestCredentials;

/// Creates a new project on the backend, including a Turnstile token if one
/// is available.
///
/// Returns [`ProjectCreationResult::Created`] on success,
/// [`ProjectCreationResult::ValidationErrors`] when the backend reports field
/// validation failures, and [`ProjectCreationResult::Failed`] for all other
/// errors.
pub async fn create_project(
    project: &ProjectData,
    turnstile_token: Option<String>,
) -> ProjectCreationResult {
    let url = projects_url();
    let mut payload = match serde_json::to_value(project) {
        Ok(json) => json,
        Err(err) => {
            return ProjectCreationResult::Failed(format!("Failed to prepare project data: {err}"));
        }
    };

    if let Some(token) = turnstile_token
        && let Some(object) = payload.as_object_mut()
    {
        object.insert(
            "_turnstile_token".to_string(),
            serde_json::Value::String(token),
        );
    }

    let body = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(err) => {
            return ProjectCreationResult::Failed(format!("Failed to prepare project data: {err}"));
        }
    };

    let request = match Request::post(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(req) => req,
        Err(err) => {
            return ProjectCreationResult::Failed(format!("Could not start the request: {err}"));
        }
    };

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            if status == 400 {
                if let Ok(parsed) = serde_json::from_str::<ValidationErrorResponse>(&text)
                    && !parsed.errors.is_empty()
                {
                    return ProjectCreationResult::ValidationErrors(parsed.errors);
                }
                return ProjectCreationResult::Failed(
                    "The project could not be created. Please check your input and try again."
                        .to_string(),
                );
            }
            if !response.ok() {
                return ProjectCreationResult::Failed(format!(
                    "Could not add the project: HTTP {status}\n{text}"
                ));
            }

            match serde_json::from_str::<ProjectData>(&text) {
                Ok(data) => ProjectCreationResult::Created(data),
                Err(err) => ProjectCreationResult::Failed(format!(
                    "Project was created, but the response could not be read: {err}"
                )),
            }
        }
        Err(err) => ProjectCreationResult::Failed(format!("Could not add the project: {err}")),
    }
}

/// Updates the title of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new title.
///
/// # Errors
///
/// Returns a [`RequestError`] when serialization fails or the backend rejects
/// the request.
pub async fn update_project_title(id: &str, title: String) -> Result<String, RequestError> {
    let url = project_url(id);
    let body = serde_json::to_string(&serde_json::json!({ "title": title })).map_err(|err| {
        RequestError::Serialize(format!("Failed to serialize title update: {err}"))
    })?;

    patch_project(&url, body, "title").await?;
    Ok(title)
}

/// Updates the tags of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new tag list.
///
/// # Errors
///
/// Returns a [`RequestError`] when serialization fails or the backend rejects
/// the request.
pub async fn update_project_tags(id: &str, tags: Vec<String>) -> Result<Vec<String>, RequestError> {
    let url = project_url(id);
    let body = serde_json::to_string(&serde_json::json!({ "tags": tags })).map_err(|err| {
        RequestError::Serialize(format!("Failed to serialize tags update: {err}"))
    })?;

    patch_project(&url, body, "tags").await?;
    Ok(tags)
}

/// Updates the description of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new description.
///
/// # Errors
///
/// Returns a [`RequestError`] when serialization fails or the backend rejects
/// the request.
pub async fn update_project_description(
    id: &str,
    description: String,
) -> Result<String, RequestError> {
    let url = project_url(id);
    let body = serde_json::to_string(&serde_json::json!({ "description": description })).map_err(
        |err| RequestError::Serialize(format!("Failed to serialize description update: {err}")),
    )?;

    patch_project(&url, body, "description").await?;
    Ok(description)
}

/// Updates the collaborators of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new collaborator id list.
///
/// # Errors
///
/// Returns a [`RequestError`] when serialization fails or the backend rejects
/// the request.
pub async fn update_project_collaborators(
    id: &str,
    collaborator_ids: Vec<String>,
) -> Result<Vec<String>, RequestError> {
    let url = project_url(id);
    let body = serde_json::to_string(&serde_json::json!({ "collaborator_ids": collaborator_ids }))
        .map_err(|err| {
            RequestError::Serialize(format!("Failed to serialize collaborator update: {err}"))
        })?;

    patch_project(&url, body, "collaborators").await?;
    Ok(collaborator_ids)
}

/// Toggles the favorite status of a project for the current user.
///
/// On success it returns the updated project.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request cannot be built, the network
/// fails, or the backend rejects the request.
pub async fn toggle_project_favorite(id: &str) -> Result<ProjectData, RequestError> {
    let url = project_favorites_url(id);
    let request = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .body("")
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build favorite toggle request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<ProjectData>(&text).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse toggled favorite response (status={status}): {err}\n{text}"
                ))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to toggle project favorite: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to toggle project favorite: {err}"
        ))),
    }
}

/// Increments the download counter for a project and returns the updated project.
///
/// On success it returns the updated project data.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request cannot be built, the network
/// fails, or the backend rejects the request.
pub async fn increment_project_downloads(id: &str) -> Result<ProjectData, RequestError> {
    let url = project_downloads_url(id);
    let request = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .body("")
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build download increment request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<ProjectData>(&text).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse download increment response (status={status}): {err}\n{text}"
                ))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to increment project downloads: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to increment project downloads: {err}"
        ))),
    }
}

/// Deletes a project's IFC model from the backend.
///
/// Returns `Ok(())` when the request succeeds.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request cannot be built, the network
/// fails, or the backend rejects the request.
pub async fn delete_project_ifc(id: &str) -> Result<(), RequestError> {
    let url = project_ifc_url(id);
    let request = Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build IFC delete request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to delete IFC model: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to delete IFC model: {err}"
        ))),
    }
}

/// Uploads an IFC model for the given project and returns its public URL.
///
/// The backend stores the file in R2 and returns the object key, which is
/// converted into a frontend download URL.
///
/// # Errors
///
/// Returns a [`RequestError`] when the form data cannot be built, the request
/// cannot be built, the network fails, or the backend rejects the request.
pub async fn upload_project_ifc(
    id: &str,
    file: web_sys::File,
    version: &str,
) -> Result<ProjectVersion, RequestError> {
    #[derive(Deserialize)]
    struct UploadResponse {
        ifc_key: String,
        version_id: String,
        file_size: i64,
        version: String,
        downloads: i64,
    }

    let url = project_ifc_url(id);
    let form = web_sys::FormData::new().map_err(|err| {
        RequestError::BuildRequest(format!("Failed to create IFC upload form data: {err:?}"))
    })?;

    form.append_with_blob_and_filename("ifc", &file, &file.name())
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to append IFC file to form data: {err:?}"))
        })?;
    form.append_with_str("version", version).map_err(|err| {
        RequestError::BuildRequest(format!("Failed to append version field: {err:?}"))
    })?;

    let request = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .body(form)
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build IFC upload request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            let upload = serde_json::from_str::<UploadResponse>(&text).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse IFC upload response (status={status}): {err}\n{text}"
                ))
            })?;
            Ok(ProjectVersion {
                id: upload.version_id,
                project_id: id.to_string(),
                filename: file.name(),
                ifc_key: upload.ifc_key.clone(),
                state: crate::metadata::version_state::VersionState::Undefined,
                created_at: crate::data::project_types::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                file_size: upload.file_size,
                version: upload.version,
                downloads: upload.downloads,
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to upload IFC model: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to upload IFC model: {err}"
        ))),
    }
}

/// Fetches the IFC versions for the given project.
///
/// Undefined versions are only included when the caller can edit the project.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails, the backend rejects the
/// request, or the response cannot be parsed.
pub async fn fetch_project_versions(id: &str) -> Result<Vec<ProjectVersion>, RequestError> {
    let url = project_versions_url(id);
    match Request::get(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Vec<ProjectVersion>>(&text).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse versions JSON from {url}: {err:?} (status={status}, body={text:?})"
                ))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch versions from {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch versions from {url}: {err:?}"
        ))),
    }
}

/// Updates the state of a single project version.
///
/// # Errors
///
/// Returns a [`RequestError`] when serialization fails, the request cannot be
/// built, the network fails, or the backend rejects the request.
pub async fn update_project_version_state(
    project_id: &str,
    version_id: &str,
    state: crate::metadata::version_state::VersionState,
) -> Result<(), RequestError> {
    let url = project_version_url(project_id, version_id);
    let body = serde_json::to_string(&serde_json::json!({
        "state": serde_json::to_string(&state).unwrap_or_default().trim_matches('"')
    }))
    .map_err(|err| {
        RequestError::Serialize(format!("Failed to serialize version state update: {err}"))
    })?;

    let request = Request::patch(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|err| {
            RequestError::BuildRequest(format!(
                "Failed to build version state update request: {err}"
            ))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to update version state: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to update version state: {err}"
        ))),
    }
}

/// Deletes a single project version.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request cannot be built, the network
/// fails, or the backend rejects the request.
pub async fn delete_project_version(
    project_id: &str,
    version_id: &str,
) -> Result<(), RequestError> {
    let url = project_version_url(project_id, version_id);
    let request = Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build version delete request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to delete version: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to delete version: {err}"
        ))),
    }
}

/// Eagerly converts a project's IFC model to GLB on the backend and caches it.
///
/// Returns `Ok(true)` when the GLB is ready to view, or `Ok(false)` when the IFC
/// model contained no renderable geometry.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request cannot be built, the network
/// fails, or the backend rejects the request with an unexpected status.
pub async fn convert_project_glb(id: &str) -> Result<bool, RequestError> {
    let url = project_glb_url(id);
    let request = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .body(web_sys::FormData::new().map_err(|err| {
            RequestError::BuildRequest(format!(
                "Failed to create GLB conversion form data: {err:?}"
            ))
        })?)
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build GLB conversion request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.status() == 422 => Ok(false),
        Ok(response) if response.ok() => Ok(true),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to convert IFC model: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to convert IFC model: {err}"
        ))),
    }
}

/// Deletes a project via `DELETE /data/projects/:id`.
///
/// Returns `Ok(())` if the deletion succeeded.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails or the backend rejects the
/// request.
pub async fn delete_project(id: &str) -> Result<(), RequestError> {
    let url = project_url(id);
    match Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to delete project: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to delete project: {err}"
        ))),
    }
}

/// Fetch projects from the backend API.
///
/// On success it returns a list of projects.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails, the backend rejects the
/// request, or the response cannot be parsed.
pub async fn fetch_projects() -> Result<Vec<ProjectData>, RequestError> {
    match Request::get(&projects_url()).send().await {
        Ok(response) if response.ok() => {
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Vec<ProjectData>>(&text).map_err(|err| {
                RequestError::Parse(format!("Failed to parse projects JSON: {err}\n{text}"))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch projects: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch projects: {err}"
        ))),
    }
}

/// Sends a `PATCH` request to the given project endpoint.
///
/// Returns `Ok(())` only when the response is successful.
async fn patch_project(url: &str, body: String, field_name: &str) -> Result<(), RequestError> {
    let request = Request::patch(url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|err| {
            RequestError::BuildRequest(format!(
                "Failed to build {field_name} update request: {err}"
            ))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to update project {field_name}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to update project {field_name}: {err}"
        ))),
    }
}
