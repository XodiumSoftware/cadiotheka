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
//!
//! Binary assets (converted GLBs, IFC models) are cached under their full
//! request URL, which is already versioned — an IFC download URL carries the
//! version id, and any write that changes a project's GLB deletes the R2
//! object. Those entries can therefore be stored for a year with `immutable`,
//! which keeps megabyte-sized model bodies out of the Worker on repeat views.

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

/// Seconds a binary asset stays fresh once it reaches a browser or the edge.
///
/// IFC download URLs carry the version id, and a project's GLB and metadata URLs
/// are purged by the writes that replace their R2 bodies, so a long TTL is safe
/// as long as every write path remembers to purge.
pub(crate) const ASSET_MAX_AGE_SECONDS: u32 = 31_536_000;

/// Returns the full URL a cached asset is keyed by.
///
/// Assets are keyed by their absolute request URL rather than by origin plus
/// path: the Cache API normalizes URL keys differently from the listing keys
/// above, and a real URL also lets a stored entry be looked up later.
pub(crate) fn asset_cache_key(req: &worker::Request) -> Option<String> {
    req.url().ok().map(|url| url.to_string())
}

/// Stores a binary asset body under its request URL.
///
/// `headers` must carry `Content-Type` and `Cache-Control`; the body is stored
/// with them so a cache hit inside a Worker can be replayed byte for byte. The
/// entry is only ever read back by a request for the same URL.
pub(crate) async fn put_asset(key: &str, body: Vec<u8>, headers: &Headers) -> Result<()> {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    let mut response = Response::from_bytes(body)?;
    for (name, value) in headers.entries() {
        response.headers_mut().set(&name, &value)?;
    }
    cache.put(key.to_string(), response).await
}

/// Returns the cached asset stored under `key`, if present.
pub(crate) async fn get_asset(key: &str) -> Option<Response> {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    match cache.get(key.to_string(), false).await {
        Ok(response) => response,
        Err(err) => {
            console_log!("cache: asset get failed for {key}: {err:?}");
            None
        }
    }
}

/// Removes the cached entry for a single asset URL.
///
/// Asset entries are keyed by the full request URL a visitor used, and a Worker
/// cannot enumerate the origins it was reached from, so callers pass the URL of
/// the request that triggered the write. Cloudflare partitions entries by the
/// `Vary: Origin` header, and a purge drops every partition of the URL.
pub(crate) async fn purge_asset(url: &str) {
    let cache = Cache::open(CACHE_NAME.to_string()).await;
    if let Err(err) = cache.delete(url.to_string(), false).await {
        console_log!("cache: asset purge failed for {url}: {err:?}");
    }
}

/// Returns the URL of a project's derived asset as seen on the API host.
///
/// Derived from `req` rather than from `public_origin` so the key matches the
/// URL the browser actually requested: that is the URL a cache hit is looked up
/// under, and purging anything else would leave the entry behind.
pub(crate) fn project_asset_url(req: &worker::Request, project_id: &str, suffix: &str) -> String {
    match req.url() {
        Ok(url) => project_asset_url_for(&url, project_id, suffix),
        Err(_) => format!("/data/projects/{project_id}/{suffix}"),
    }
}

/// Builds the asset URL for `project_id` from any URL on the API host.
///
/// Keeps the request's origin and scheme, then replaces the path so the result
/// is the exact URL a viewer requests for the artifact.
pub(crate) fn project_asset_url_for(base: &url::Url, project_id: &str, suffix: &str) -> String {
    let mut url = base.clone();
    url.set_path(&format!("/data/projects/{project_id}/{suffix}"));
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
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
    fn asset_urls_build_canonical_paths() -> Result<(), url::ParseError> {
        let base = url::Url::parse("https://api.cadiotheka.com/data/projects/proj-1/glb?x=1")?;
        assert_eq!(
            project_asset_url_for(&base, "proj-1", "glb"),
            "https://api.cadiotheka.com/data/projects/proj-1/glb"
        );
        assert_eq!(
            project_asset_url_for(&base, "proj-1", "glb-metadata"),
            "https://api.cadiotheka.com/data/projects/proj-1/glb-metadata"
        );
        Ok(())
    }

    #[test]
    fn project_asset_url_keeps_request_origin() -> Result<(), url::ParseError> {
        let base = url::Url::parse("http://localhost:8787/data/projects/p/glb")?;
        assert_eq!(
            project_asset_url_for(&base, "other", "glb"),
            "http://localhost:8787/data/projects/other/glb"
        );
        Ok(())
    }

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
