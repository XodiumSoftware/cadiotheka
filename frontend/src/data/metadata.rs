//! Metadata (tags and platforms) fetched from the backend.

use crate::data::error::RequestError;
use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use crate::utils::api_url;
use gloo_net::http::Request;

/// Fetches the available content tags from `/data/tags`.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails or the response is not
/// valid JSON.
pub async fn fetch_tags() -> Result<Vec<Tag>, RequestError> {
    fetch_metadata("/tags").await
}

/// Fetches the available CAD platforms from `/data/platforms`.
///
/// # Errors
///
/// Returns a [`RequestError`] when the request fails or the response is not
/// valid JSON.
pub async fn fetch_platforms() -> Result<Vec<Platform>, RequestError> {
    fetch_metadata("/platforms").await
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
