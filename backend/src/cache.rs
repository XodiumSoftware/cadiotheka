//! Edge caching for the public, anonymous JSON read endpoints.
//!
//! The project and account listings are identical for every visitor, so they
//! can be served straight from the Cloudflare cache instead of reading D1 on
//! every request. Writes purge the affected listing so the change is visible
//! on the next request.
//!
//! Only bodies are cached, never headers. The Cache API matches a plain fetch
//! of the request URL as readily as a Worker lookup, and Cloudflare's own
//! cache sits in front of this Worker, so a stored response carrying CORS is
//! either replayed to a different origin or returned as a cache hit whose
//! headers the fetch handler never gets to rewrite.
//!
//! Invalidation is best-effort, so entries carry a short `max-age` to bound
//! how long a body can outlive the write that should have purged it.

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
/// per minute. The Worker `put`s without a TTL, so this header is what makes
/// the edge treat the entry as expired.
pub(crate) const LIST_CACHE_MAX_AGE_SECONDS: u32 = 60;

/// Cache key for a listing page served to a specific origin.
///
/// Page size and offset live in the query string, so each page gets its own
/// entry rather than every `?limit=`/`?offset=` combination sharing one
/// response. The origin scopes the entry to the visitor's public origin, which
/// keeps local development from colliding with production.
pub(crate) fn list_cache_key(origin: &str, path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) if !query.is_empty() => format!("{origin}{path}?{query}"),
        _ => format!("{origin}{path}"),
    }
}

/// Returns the cached body for `key`, if present.
pub(crate) async fn get(key: &str) -> Option<String> {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    match cache.get(key.to_string(), true).await {
        Ok(Some(mut response)) => response.text().await.ok(),
        Ok(None) => None,
        Err(err) => {
            console_log!("cache: get failed for {key}: {err:?}");
            None
        }
    }
}

/// Stores a listing body under `key`.
///
/// The body is stored without headers so a later visitor cannot receive
/// headers built for someone else; [`list_response`] decorates whatever is
/// served.
pub(crate) async fn put(key: &str, body: &str) -> Result<()> {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    cache.put(key.to_string(), Response::ok(body)?).await
}

/// Removes a single cached entry, logging and ignoring failures.
pub(crate) async fn delete(key: &str) {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    if let Err(err) = cache.delete(key.to_string(), true).await {
        console_log!("cache: delete failed for {key}: {err:?}");
    }
}

/// Builds a listing response with the JSON and caching headers.
///
/// Every listing response goes through here — cache hit or fresh — so a body
/// taken from the edge cache still reaches the browser with the headers it
/// was stored without.
pub(crate) fn list_response(body: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set(
        "Cache-Control",
        &format!("public, max-age={LIST_CACHE_MAX_AGE_SECONDS}"),
    )?;
    Ok(Response::ok(body)?.with_headers(headers))
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
}
