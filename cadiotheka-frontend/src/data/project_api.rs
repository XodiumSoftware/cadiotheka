use crate::data::project_types::{
    IconUrl, ProjectCreationResult, ProjectData, ProjectPatch, ValidationErrorResponse,
    icon_src_from_key, ifc_src_from_key,
};
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
            leptos::web_sys::console::error_1(
                &format!("Failed to serialize project payload: {err:?}").into(),
            );
            return ProjectCreationResult::Failed(
                "Failed to prepare project data. Please try again.".to_string(),
            );
        }
    };

    let request = match Request::post(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build project creation request: {err:?}").into(),
            );
            return ProjectCreationResult::Failed(
                "Could not start the request. Please try again.".to_string(),
            );
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
                leptos::web_sys::console::error_1(
                    &format!("Failed to create project: HTTP {status}\n{text}").into(),
                );
                return ProjectCreationResult::Failed(
                    "Could not add the project. Please try again.".to_string(),
                );
            }

            match serde_json::from_str::<ProjectData>(&text) {
                Ok(data) => ProjectCreationResult::Created(data),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse created project response (status={status}): {err:?}\n{text}"
                        )
                        .into(),
                    );
                    ProjectCreationResult::Failed(
                        "Project was created, but the response could not be read.".to_string(),
                    )
                }
            }
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to create project: {err:?}").into());
            ProjectCreationResult::Failed(
                "Could not add the project. Please try again.".to_string(),
            )
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

/// Updates the tags of an existing project via `PATCH /data/projects/:id`.
///
/// On success it returns the new tag list; on failure it logs to the console and
/// returns `None`.
pub async fn update_project_tags(
    id: &str,
    tags: Vec<crate::metadata::tags::Tag>,
) -> Option<Vec<crate::metadata::tags::Tag>> {
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
    supported_platforms: Vec<crate::metadata::platforms::Platform>,
) -> Option<Vec<crate::metadata::platforms::Platform>> {
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

/// Updates the description of an existing project via `PATCH /data/projects/:id`.
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

/// Updates the collaborators of an existing project via `PATCH /data/projects/:id`.
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

    match Request::patch(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(request) => match request.send().await {
            Ok(response) => {
                if response.ok() {
                    true
                } else {
                    let status = response.status();
                    leptos::web_sys::console::error_1(
                        &format!("Failed to update project: HTTP {status}").into(),
                    );
                    false
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

/// Toggles the favorite status of a project for the current user.
///
/// On success it returns the updated project; on failure it logs to the console
/// and returns `None`.
pub async fn toggle_project_favorite(id: &str) -> Option<ProjectData> {
    let url = api_url(&format!("/projects/{id}/favorites"));
    let request = match Request::post(&url)
        .credentials(RequestCredentials::Include)
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

/// Uploads a project icon and returns its public URL.
pub async fn upload_project_icon(id: &str, file: web_sys::File) -> Option<IconUrl> {
    #[derive(Deserialize)]
    struct UploadResponse {
        icon_key: String,
    }

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

    let request = match Request::post(&url)
        .credentials(RequestCredentials::Include)
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

/// Deletes a project's IFC model from the backend.
///
/// Returns `true` when the request succeeds.
pub async fn delete_project_ifc(id: &str) -> bool {
    let url = api_url(&format!("/projects/{id}/ifc"));
    let request = match Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body("{}")
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build IFC delete request: {err:?}").into(),
            );
            return false;
        }
    };

    match request.send().await {
        Ok(response) => {
            if !response.ok() {
                let status = response.status();
                leptos::web_sys::console::error_1(
                    &format!("Failed to delete IFC model: HTTP {status}").into(),
                );
                return false;
            }
            true
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to delete IFC model: {err:?}").into(),
            );
            false
        }
    }
}

/// Uploads an IFC model for the given project and returns its public URL.
///
/// The backend stores the file in R2 and returns the object key, which is
/// converted into a frontend download URL.
pub async fn upload_project_ifc(id: &str, file: web_sys::File) -> Option<String> {
    #[derive(Deserialize)]
    struct UploadResponse {
        ifc_key: String,
    }

    let url = api_url(&format!("/projects/{id}/ifc"));
    let form = match web_sys::FormData::new() {
        Ok(form) => form,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to create IFC upload form data: {err:?}").into(),
            );
            return None;
        }
    };

    if let Err(err) = form.append_with_blob_and_filename("ifc", &file, &file.name()) {
        leptos::web_sys::console::error_1(
            &format!("Failed to append IFC file to form data: {err:?}").into(),
        );
        return None;
    }

    let request = match Request::post(&url)
        .credentials(RequestCredentials::Include)
        .body(form)
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build IFC upload request: {err:?}").into(),
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
                    &format!("Failed to upload IFC model: HTTP {status}\n{text}").into(),
                );
                return None;
            }

            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<UploadResponse>(&text) {
                Ok(upload) => Some(ifc_src_from_key(&upload.ifc_key)),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse IFC upload response (status={status}): {err:?}\n{text}"
                        )
                        .into(),
                    );
                    None
                }
            }
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to upload IFC model: {err:?}").into(),
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
    match Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) => {
            if response.ok() {
                true
            } else {
                let status = response.status();
                leptos::web_sys::console::error_1(
                    &format!("Failed to delete project: HTTP {status}").into(),
                );
                false
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
    match Request::get(&api_url("/projects")).send().await {
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

/// Sends a `PATCH` request to the given project endpoint.
///
/// Logs failures under the provided field name and returns `Some(())` only when
/// the response is successful.
async fn patch_project(url: &str, body: String, field_name: &str) -> Option<()> {
    let request = match Request::patch(url)
        .credentials(RequestCredentials::Include)
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
