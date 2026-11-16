//! Edge caching for the public, anonymous JSON read endpoints.
//!
//! The project and account listings are identical for every visitor, so they
//! can be served straight from the Cloudflare cache instead of reading D1 on
//! every request. Writes delete the affected listing so the change is visible
//! on the next request.
//!
//! Two details make a cached response safe to replay:
//!
//! - The Cache API stores a response verbatim, so `Access-Control-Allow-Origin`
//!   is baked in at write time and the key includes the requesting origin. A
//!   response built for one origin is never served to another.
//! - Invalidation is best-effort, so entries carry a short `max-age` to bound
//!   how long a body can outlive the write that should have purged it.

use worker::{Cache, Headers, Response, Result, console_log};

/// Cache instance name. Cloudflare only honors the `default` cache for
/// Worker-originated `put`/`get` calls.
const CACHE_NAME: &str = "default";

/// Path of the public project listing.
pub(crate) const PROJECTS_LIST_PATH: &str = "/data/projects";
/// Path of the public account listing.
pub(crate) const ACCOUNTS_LIST_PATH: &str = "/data/accounts";

/// Seconds a cached listing stays fresh in the edge cache.
///
/// Bounds staleness when a purge fails, at the cost of one D1 read per page
/// per minute.
pub(crate) const LIST_CACHE_MAX_AGE_SECONDS: u32 = 60;

/// Cache key for a specific listing page served to a specific origin.
///
/// Page size and offset live in the query string, so each page gets its own
/// entry rather than every `?limit=`/`?offset=` combination sharing one
/// response. The origin keeps CORS-baked responses from crossing origins.
pub(crate) fn list_cache_key(origin: &str, path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) if !query.is_empty() => format!("{origin}{path}?{query}"),
        _ => format!("{origin}{path}"),
    }
}

/// Returns the cached response for `key`, if present.
pub(crate) async fn get(key: &str) -> Option<Response> {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    match cache.get(key.to_string(), true).await {
        Ok(response) => response,
        Err(err) => {
            console_log!("cache: get failed for {key}: {err:?}");
            None
        }
    }
}

/// Stores a response in the edge cache under `key`.
pub(crate) async fn put(key: &str, response: Response) -> Result<()> {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    cache.put(key.to_string(), response).await
}

/// Serializes a listing body and stores it, with CORS and cache headers baked
/// in, so a later [`get`] can be returned without further rewriting.
///
/// Failures are logged and ignored: a miss only costs a D1 read.
pub(crate) async fn store(key: &str, body: &str, origin: &str) {
    let headers = cache_headers();
    if let Err(err) = crate::cors::apply_cors_headers(&headers, origin) {
        console_log!("cache: failed to set CORS headers: {err:?}");
        return;
    }

    let response = match Response::ok(body) {
        Ok(response) => response.with_headers(headers),
        Err(err) => {
            console_log!("cache: failed to build response: {err:?}");
            return;
        }
    };

    if let Err(err) = put(key, response).await {
        console_log!("cache: put failed for {key}: {err:?}");
    }
}

/// Builds the headers for a cacheable JSON listing response.
pub(crate) fn cache_headers() -> Headers {
    let headers = Headers::new();
    if let Err(err) = headers.set("Content-Type", "application/json") {
        console_log!("cache: failed to set Content-Type: {err:?}");
    }
    let cache_control = format!("public, max-age={LIST_CACHE_MAX_AGE_SECONDS}");
    if let Err(err) = headers.set("Cache-Control", &cache_control) {
        console_log!("cache: failed to set Cache-Control: {err:?}");
    }
    headers
}

/// Removes a single cached entry, logging and ignoring failures.
pub(crate) async fn delete(key: &str) {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    if let Err(err) = cache.delete(key.to_string(), true).await {
        console_log!("cache: delete failed for {key}: {err:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_cache_key_includes_query_for_paging() {
        let key = list_cache_key(
            "https://api.cadiotheka.com",
            PROJECTS_LIST_PATH,
            Some("limit=10&offset=20"),
        );
        assert_eq!(
            key,
            "https://api.cadiotheka.com/data/projects?limit=10&offset=20"
        );
    }

    #[test]
    fn list_cache_key_omits_empty_query() {
        assert_eq!(
            list_cache_key("https://api.cadiotheka.com", ACCOUNTS_LIST_PATH, None),
            "https://api.cadiotheka.com/data/accounts"
        );
        assert_eq!(
            list_cache_key("https://api.cadiotheka.com", ACCOUNTS_LIST_PATH, Some("")),
            "https://api.cadiotheka.com/data/accounts"
        );
    }

    #[test]
    fn list_cache_key_differs_between_pages() {
        assert_ne!(
            list_cache_key("https://a", PROJECTS_LIST_PATH, None),
            list_cache_key("https://a", PROJECTS_LIST_PATH, Some("offset=100"))
        );
    }

    #[test]
    fn list_cache_key_differs_between_origins() {
        assert_ne!(
            list_cache_key("https://api.cadiotheka.com", PROJECTS_LIST_PATH, None),
            list_cache_key("http://localhost:8787", PROJECTS_LIST_PATH, None)
        );
    }
}
