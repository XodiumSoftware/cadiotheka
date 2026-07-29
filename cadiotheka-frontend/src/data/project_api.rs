#![allow(clippy::missing_errors_doc)]

use crate::data::error::RequestError;
use crate::data::project_types::{
    IconUrl, ProjectCreationResult, ProjectData, ProjectPatch, ValidationErrorResponse,
    icon_src_from_key, ifc_src_from_key,
};
use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use crate::utils::api_url;
use gloo_net::http::Request;
use serde::Deserialize;
use web_sys::RequestCredentials;

/// Creates a new project on the backend.
///
/// Returns [`ProjectCreationResult::Created`] on success,
/// [`ProjectCreationResult::ValidationErrors`] when the backend reports field
/// validation failures, and [`ProjectCreationResult::Failed`] for all other
/// errors.
pub async fn create_project(project: &ProjectData) -> ProjectCreationResult {
    let url = api_url("/projects");
    let body = match serde_json::to_string(project) {
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
/// On success it returns the new title; on failure it returns a [`RequestError`].
pub async fn update_project_title(id: &str, title: String) -> Result<String, RequestError> {
    let url = api_url(&format!("/projects/{id}"));
    let body = serde_json::to_string(&serde_json::json!({ "title": title })).map_err(|err| {
        RequestError::Serialize(format!("Failed to serialize title update: {err}"))
    })?;

    patch_project(&url, body, "title").await?;
    Ok(title)
}

/// Updates the tags of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new tag list; on failure it returns a [`RequestError`].
pub async fn update_project_tags(id: &str, tags: Vec<Tag>) -> Result<Vec<Tag>, RequestError> {
    let url = api_url(&format!("/projects/{id}"));
    let body = serde_json::to_string(&serde_json::json!({ "tags": tags })).map_err(|err| {
        RequestError::Serialize(format!("Failed to serialize tags update: {err}"))
    })?;

    patch_project(&url, body, "tags").await?;
    Ok(tags)
}

/// Updates the supported platforms of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new platform list; on failure it returns a [`RequestError`].
pub async fn update_project_platforms(
    id: &str,
    supported_platforms: Vec<Platform>,
) -> Result<Vec<Platform>, RequestError> {
    let url = api_url(&format!("/projects/{id}"));
    let body =
        serde_json::to_string(&serde_json::json!({ "supported_platforms": supported_platforms }))
            .map_err(|err| {
            RequestError::Serialize(format!(
                "Failed to serialize supported platforms update: {err}"
            ))
        })?;

    patch_project(&url, body, "supported platforms").await?;
    Ok(supported_platforms)
}

/// Updates the description of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new description; on failure it returns a [`RequestError`].
pub async fn update_project_description(
    id: &str,
    description: String,
) -> Result<String, RequestError> {
    let url = api_url(&format!("/projects/{id}"));
    let body = serde_json::to_string(&serde_json::json!({ "description": description })).map_err(
        |err| RequestError::Serialize(format!("Failed to serialize description update: {err}")),
    )?;

    patch_project(&url, body, "description").await?;
    Ok(description)
}

/// Updates the collaborators of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new collaborator id list; on failure it returns a
/// [`RequestError`].
pub async fn update_project_collaborators(
    id: &str,
    collaborator_ids: Vec<String>,
) -> Result<Vec<String>, RequestError> {
    let url = api_url(&format!("/projects/{id}"));
    let body = serde_json::to_string(&serde_json::json!({ "collaborator_ids": collaborator_ids }))
        .map_err(|err| {
            RequestError::Serialize(format!("Failed to serialize collaborator update: {err}"))
        })?;

    patch_project(&url, body, "collaborators").await?;
    Ok(collaborator_ids)
}

/// Applies a partial update to an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns `true`; on failure it returns a [`RequestError`].
pub async fn update_project(id: &str, patch: ProjectPatch) -> Result<(), RequestError> {
    let url = api_url(&format!("/projects/{id}"));
    let body = serde_json::to_string(&patch).map_err(|err| {
        RequestError::Serialize(format!("Failed to serialize project update: {err}"))
    })?;

    let request = Request::patch(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build project update request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to update project: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to update project: {err}"
        ))),
    }
}

/// Toggles the favorite status of a project for the current user.
///
/// On success it returns the updated project; on failure it returns a
/// [`RequestError`].
pub async fn toggle_project_favorite(id: &str) -> Result<ProjectData, RequestError> {
    let url = api_url(&format!("/projects/{id}/favorites"));
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
/// On success it returns the updated project data; on failure it returns a
/// [`RequestError`].
pub async fn increment_project_downloads(id: &str) -> Result<ProjectData, RequestError> {
    let url = api_url(&format!("/projects/{id}/downloads"));
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

/// Uploads a project icon and returns its public URL.
pub async fn upload_project_icon(id: &str, file: web_sys::File) -> Result<IconUrl, RequestError> {
    #[derive(Deserialize)]
    struct UploadResponse {
        icon_key: String,
    }

    let url = api_url(&format!("/projects/{id}/icon"));
    let form = web_sys::FormData::new().map_err(|err| {
        RequestError::BuildRequest(format!("Failed to create icon upload form data: {err:?}"))
    })?;

    form.append_with_blob_and_filename("icon", &file, &file.name())
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to append icon file to form data: {err:?}"))
        })?;

    let request = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .body(form)
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build icon upload request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            let upload = serde_json::from_str::<UploadResponse>(&text).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse project icon upload response (status={status}): {err}\n{text}"
                ))
            })?;
            Ok(icon_src_from_key(&upload.icon_key))
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to upload project icon: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to upload project icon: {err}"
        ))),
    }
}

/// Deletes a project's IFC model from the backend.
///
/// Returns `Ok(())` when the request succeeds.
pub async fn delete_project_ifc(id: &str) -> Result<(), RequestError> {
    let url = api_url(&format!("/projects/{id}/ifc"));
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
pub async fn upload_project_ifc(id: &str, file: web_sys::File) -> Result<String, RequestError> {
    #[derive(Deserialize)]
    struct UploadResponse {
        ifc_key: String,
    }

    let url = api_url(&format!("/projects/{id}/ifc"));
    let form = web_sys::FormData::new().map_err(|err| {
        RequestError::BuildRequest(format!("Failed to create IFC upload form data: {err:?}"))
    })?;

    form.append_with_blob_and_filename("ifc", &file, &file.name())
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to append IFC file to form data: {err:?}"))
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
            Ok(ifc_src_from_key(&upload.ifc_key))
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

/// Deletes a project via `DELETE /data/projects/:id`.
///
/// Returns `Ok(())` if the deletion succeeded.
pub async fn delete_project(id: &str) -> Result<(), RequestError> {
    let url = api_url(&format!("/projects/{id}"));
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
/// Returns a list of projects on success or a [`RequestError`] describing what
/// went wrong.
pub async fn fetch_projects() -> Result<Vec<ProjectData>, RequestError> {
    match Request::get(&api_url("/projects")).send().await {
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
