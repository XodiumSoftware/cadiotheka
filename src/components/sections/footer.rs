use crate::i18n::{t_string, use_i18n};
use crate::utils::clean_sha;
use leptos::prelude::*;

const GIT_SHA: &str = env!("GIT_SHA");
const GITHUB_REPO_URL: &str = "https://github.com/XodiumSoftware/cadiotheka/commit";

#[component]
pub fn Footer() -> impl IntoView {
    let i18n = use_i18n();
    let current_year = js_sys::Date::new_0().get_full_year();
    let commit_url = if GIT_SHA == "unknown" {
        None
    } else {
        Some(format!("{GITHUB_REPO_URL}/{}", clean_sha(GIT_SHA)))
    };

    view! {
        <footer class="text-base-content px-6 lg:px-8 py-4">
            <div class="mx-auto max-w-7xl flex justify-center items-center gap-4">
                <aside class="text-center" style="container-type: inline-size">
                    <div class="flex flex-col md:flex-row md:items-center gap-1 md:gap-0 justify-center">
                        <p class="font-bold whitespace-nowrap max-md:text-[clamp(0.5rem,3.1cqw,1rem)]">
                            {format!("© {current_year} ")}
                            <a href="/" class="link link-hover link-primary">
                                {t_string!(i18n, footer.company_name)}
                            </a>
                            {t_string!(i18n, footer.tagline)}
                        </p>
                        {commit_url.map(|url| view! {
                            <a
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link link-hover text-base-content/50 hover:link-primary font-mono md:ml-1"
                            >
                                {format!("Build: #{GIT_SHA}")}
                            </a>
                        })}
                    </div>
                </aside>
            </div>
        </footer>
    }
}
