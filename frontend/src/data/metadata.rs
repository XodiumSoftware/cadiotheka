//! Metadata (tags) fetched from the backend.

use crate::data::error::RequestError;
use crate::metadata::tags::Tag;
use crate::utils::api_url;
use gloo_net::http::Request;
use web_sys::RequestCredentials;

/// Fetches the available content tags from `/data/tags`.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails or the response is not
/// valid JSON.
pub async fn fetch_tags() -> Result<Vec<Tag>, RequestError> {
    fetch_metadata("/tags").await
}

/// Fetches a metadata collection from the given relative endpoint.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails or the response is not
/// valid JSON.
async fn fetch_metadata<T: serde::de::DeserializeOwned>(
    path: &str,
) -> Result<Vec<T>, RequestError> {
    match Request::get(&api_url(path)).send().await {
        Ok(response) if response.ok() => {
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Vec<T>>(&text).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse metadata JSON from {path}: {err}\n{text}"
                ))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch metadata from {path}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch metadata from {path}: {err}"
        ))),
    }
}

/// Creates a new content tag on the backend.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails or the response is not
/// valid JSON.
pub async fn create_tag(id: String, label: String, color: String) -> Result<Tag, RequestError> {
    let body =
        serde_json::to_string(&serde_json::json!({ "id": id, "label": label, "color": color }))
            .map_err(|err| {
                RequestError::Serialize(format!("Failed to serialize tag create: {err}"))
            })?;

    let request = Request::post(&api_url("/tags"))
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build tag create request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => {
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Tag>(&text).map_err(|err| {
                RequestError::Parse(format!("Failed to parse created tag JSON: {err}\n{text}"))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to create tag: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to create tag: {err}"
        ))),
    }
}

/// Updates an existing content tag's label and color.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails or the response is not
/// valid JSON.
pub async fn update_tag(id: &str, label: String, color: String) -> Result<Tag, RequestError> {
    let body = serde_json::to_string(&serde_json::json!({ "label": label, "color": color }))
        .map_err(|err| RequestError::Serialize(format!("Failed to serialize tag update: {err}")))?;

    let request = Request::put(&api_url(&format!("/tags/{id}")))
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build tag update request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => {
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Tag>(&text).map_err(|err| {
                RequestError::Parse(format!("Failed to parse updated tag JSON: {err}\n{text}"))
            })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to update tag: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to update tag: {err}"
        ))),
    }
}

/// Deletes a content tag on the backend.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails.
pub async fn delete_tag(id: &str) -> Result<(), RequestError> {
    match Request::delete(&api_url(&format!("/tags/{id}")))
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
                body: format!("Failed to delete tag: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to delete tag: {err}"
        ))),
    }
}
