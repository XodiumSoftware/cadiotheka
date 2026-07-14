use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const ORG: &str = "XodiumSoftware";
const API_BASE: &str = "https://api.github.com";
const CACHE_TTL_MS: f64 = 5.0 * 60.0 * 1000.0;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_MS: u64 = 1000;
const PER_PAGE: usize = 100;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Repo {
    pub name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub fork: bool,
    pub has_pages: bool,
    pub topics: Vec<String>,
}

fn cache_get<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let ts: f64 = storage.get_item(&format!("{key}:ts")).ok()??.parse().ok()?;
    if js_sys::Date::now() - ts > CACHE_TTL_MS {
        let _ = storage.remove_item(key);
        let _ = storage.remove_item(&format!("{key}:ts"));
        return None;
    }
    let raw = storage.get_item(key).ok()??;
    serde_json::from_str(&raw).ok()
}

fn cache_set<T: Serialize>(key: &str, data: &T) {
    let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) else {
        return;
    };
    if let Ok(json) = serde_json::to_string(data) {
        let _ = storage.set_item(key, &json);
        let _ = storage.set_item(&format!("{key}:ts"), &js_sys::Date::now().to_string());
    }
}

fn format_api_error(status: u16) -> String {
    match status {
        403 => "GitHub API rate limit reached. Please try again later.".to_string(),
        404 => "Organization or resource not found on GitHub.".to_string(),
        500 | 502 | 503 | 504 => {
            "GitHub is temporarily unavailable. Please try again later.".to_string()
        }
        _ => format!("Failed to load data from GitHub (status {status}). Please try again later."),
    }
}

fn format_network_error() -> String {
    "Could not reach GitHub. Please check your network connection and try again.".to_string()
}

async fn fetch<T: for<'de> Deserialize<'de> + Serialize>(endpoint: &str) -> Result<T, String> {
    let cache_key = format!("xodium:{endpoint}");

    if let Some(cached) = cache_get::<T>(&cache_key) {
        return Ok(cached);
    }

    let url = format!("{API_BASE}{endpoint}");
    let mut last_err = String::new();
    let mut network_failure_count: u32 = 0;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            gloo_timers::future::sleep(Duration::from_millis(RETRY_BASE_MS << (attempt - 1))).await;
        }

        let response = match Request::get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                network_failure_count += 1;
                last_err = e.to_string();
                continue;
            }
        };

        if response.status() >= 500 {
            last_err = format_api_error(response.status());
            continue;
        }

        if !response.ok() {
            return Err(format_api_error(response.status()));
        }

        let data = response.json::<T>().await.map_err(|e| e.to_string())?;
        cache_set(&cache_key, &data);
        return Ok(data);
    }

    if network_failure_count > 0 {
        Err(format_network_error())
    } else {
        Err(last_err)
    }
}

async fn fetch_all<T: for<'de> Deserialize<'de> + Serialize>(
    endpoint: &str,
) -> Result<Vec<T>, String> {
    let sep = if endpoint.contains('?') { '&' } else { '?' };
    let mut all = Vec::new();
    let mut page = 1u32;
    loop {
        let page_endpoint = format!("{endpoint}{sep}page={page}&per_page={PER_PAGE}");
        let items: Vec<T> = fetch(&page_endpoint).await?;
        let done = items.len() < PER_PAGE;
        all.extend(items);
        if done {
            break;
        }
        page += 1;
    }
    Ok(all)
}

pub async fn fetch_repos() -> Result<Vec<Repo>, String> {
    let mut repos = fetch_all::<Repo>(&format!("/orgs/{ORG}/repos?type=public")).await?;
    repos.retain(|r| !r.fork);
    repos.sort_by_key(|b| std::cmp::Reverse(b.stargazers_count));
    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_constants() {
        assert_eq!(ORG, "XodiumSoftware");
        assert_eq!(API_BASE, "https://api.github.com");
        assert_eq!(CACHE_TTL_MS, 5.0 * 60.0 * 1000.0);
        assert_eq!(MAX_RETRIES, 3);
        assert_eq!(RETRY_BASE_MS, 1000);
        assert_eq!(PER_PAGE, 100);
    }

    #[wasm_bindgen_test]
    fn test_retry_delay_calculation() {
        assert_eq!(RETRY_BASE_MS, 1000);
        assert_eq!(RETRY_BASE_MS << 1, 2000);
        assert_eq!(RETRY_BASE_MS << 2, 4000);
    }

    #[wasm_bindgen_test]
    fn test_repo_deserialization() {
        let json = r#"{
            "name": "test-repo",
            "description": "A test repository",
            "html_url": "https://github.com/XodiumSoftware/test-repo",
            "language": "Rust",
            "stargazers_count": 42,
            "fork": false,
            "has_pages": true,
            "topics": ["cad", "cli", "rust"]
        }"#;

        let repo: Repo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.description, Some("A test repository".to_string()));
        assert_eq!(repo.language, Some("Rust".to_string()));
        assert_eq!(repo.stargazers_count, 42);
        assert!(!repo.fork);
        assert!(repo.has_pages);
        assert_eq!(repo.topics, vec!["cad", "cli", "rust"]);
    }

    #[wasm_bindgen_test]
    fn test_repo_filtering_and_sorting() {
        let mut repos = vec![
            Repo {
                name: "repo-a".to_string(),
                description: None,
                html_url: "https://github.com/XodiumSoftware/repo-a".to_string(),
                language: Some("Rust".to_string()),
                stargazers_count: 10,
                fork: false,
                has_pages: false,
                topics: vec!["cad".to_string()],
            },
            Repo {
                name: "repo-b".to_string(),
                description: None,
                html_url: "https://github.com/XodiumSoftware/repo-b".to_string(),
                language: Some("Python".to_string()),
                stargazers_count: 50,
                fork: false,
                has_pages: false,
                topics: vec!["python".to_string()],
            },
            Repo {
                name: "repo-c".to_string(),
                description: None,
                html_url: "https://github.com/XodiumSoftware/repo-c".to_string(),
                language: Some("Go".to_string()),
                stargazers_count: 30,
                fork: true, // This should be filtered out
                has_pages: false,
                topics: vec![],
            },
        ];

        repos.retain(|r| !r.fork);
        repos.sort_by_key(|b| std::cmp::Reverse(b.stargazers_count));

        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].name, "repo-b");
        assert_eq!(repos[1].name, "repo-a");
    }

    #[wasm_bindgen_test]
    async fn test_cache_operations() {
        let endpoint = "/orgs/XodiumSoftware/repos";
        let cache_key = format!("xodium:{endpoint}");
        assert_eq!(cache_key, "xodium:/orgs/XodiumSoftware/repos");
    }
}
