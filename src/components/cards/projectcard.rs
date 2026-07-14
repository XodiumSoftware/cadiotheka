use crate::components::ui::cornerframe::CornerFrame;
use crate::github::Repo;
use crate::i18n::{t, t_string, use_i18n};
use crate::utils::language_color;
use leptos::prelude::*;

#[derive(Clone)]
pub struct ProjectCardProperties {
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub has_pages: bool,
    pub topics: Vec<String>,
}

impl From<Repo> for ProjectCardProperties {
    fn from(repo: Repo) -> Self {
        let description = repo
            .description
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| "(No description)".to_string());
        Self {
            title: repo.name,
            description,
            link: Some(repo.html_url),
            language: repo.language,
            stargazers_count: repo.stargazers_count,
            has_pages: repo.has_pages,
            topics: repo.topics,
        }
    }
}

#[component]
fn LanguageCircle(language: String, color: &'static str) -> impl IntoView {
    view! { <span class=format!("badge badge-sm {} mr-1", color) title=language /> }
}

#[component]
fn StarIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
        </svg>
    }
}

#[component]
fn DocsIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
        </svg>
    }
}

#[component]
pub fn ProjectCard(props: ProjectCardProperties) -> impl IntoView {
    let i18n = use_i18n();
    let link = props.link.clone().unwrap_or_else(|| "#".to_string());
    let stargazers_url = format!("{link}/stargazers");
    let stars = props.stargazers_count;
    let docs_url = props
        .has_pages
        .then(|| format!("https://{}.xodium.org", props.title.to_lowercase()));
    let language_badge = props.language.as_ref().map(|language| {
        let color = language_color(language);
        let language = language.clone();
        view! {
            <>
                <LanguageCircle language=language.clone() color=color />
                <span>{language}</span>
            </>
        }
    });

    view! {
        <article class="btn-lift hover:border-primary block h-full p-2">
            <CornerFrame style="square" class="h-full">
                <div class="card bg-ghost h-full rounded-none">
                    <div class="card-body">
                        <h2 class="card-title text-primary">
                            <img
                                src="/icons/github-repo.svg"
                                alt=move || t_string!(i18n, projects.card.repo_alt)
                                class="w-5 h-5 text-base-content/60 invert"
                            />
                            <a
                                href=link
                                target="_blank"
                                rel="noopener noreferrer"
                                class="hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-primary rounded"
                            >
                                {props.title}
                            </a>
                        </h2>

                        <div class="flex flex-wrap gap-1 mb-2">
                            {props
                                .topics
                                .iter()
                                .map(|topic| {
                                    view! {
                                        <span class="badge badge-xs badge-outline text-base-content/60">
                                            {topic.clone()}
                                        </span>
                                    }
                                })
                                .collect_view()}
                        </div>

                        <p class="text-base-content/70 flex-grow">{props.description}</p>

                        <div class="card-actions justify-between items-center mt-auto">
                            <div class="flex items-center gap-1 text-base-content/60 text-sm">
                                {language_badge}
                            </div>
                            <div class="flex items-center gap-3">
                            {docs_url.map(|url| view! {
                                <a
                                    href=url
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="flex items-center gap-1 text-base-content/60 hover:text-primary text-sm transition-colors"
                                    title="Documentation"
                                >
                                    <DocsIcon />
                                    <span>{t!(i18n, projects.card.docs)}</span>
                                </a>
                            })}
                            {if stars > 0 {
                                view! {
                                    <a
                                        href=stargazers_url
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="flex items-center gap-1 text-base-content/60 hover:text-primary text-sm transition-colors"
                                    >
                                        <StarIcon />
                                        <span>{stars}</span>
                                    </a>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="flex items-center gap-1 text-base-content/60 text-sm">
                                        <StarIcon />
                                        <span>{stars}</span>
                                    </div>
                                }
                                    .into_any()
                            }}
                            </div>
                        </div>
                    </div>
                </div>
            </CornerFrame>
        </article>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_language_color() {
        assert_eq!(language_color("Rust"), "bg-[#dea584]");
        assert_eq!(language_color("TypeScript"), "bg-[#3178c6]");
        assert_eq!(language_color("Python"), "bg-[#3572A5]");
        assert_eq!(language_color("UnknownLang"), "bg-base-content/50");
    }

    #[wasm_bindgen_test]
    fn test_project_card_properties() {
        let props = ProjectCardProperties {
            title: "test-repo".to_string(),
            description: "A test repository".to_string(),
            link: Some("https://github.com/test".to_string()),
            language: Some("Rust".to_string()),
            stargazers_count: 42,
            has_pages: true,
            topics: vec!["cad".to_string(), "cli".to_string()],
        };
        assert_eq!(props.title, "test-repo");
        assert_eq!(props.topics.len(), 2);
        assert!(props.has_pages);
    }
}
