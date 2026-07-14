use crate::i18n::{t_string, use_i18n};
use crate::utils::clean_sha;
use leptos::prelude::*;

struct FooterLink {
    href: &'static str,
    label_key: &'static str,
    is_external: bool,
}

const FOOTER_LINKS: &[FooterLink] = &[
    FooterLink {
        href: "https://github.com/XodiumSoftware",
        label_key: "footer.about",
        is_external: true,
    },
    FooterLink {
        href: "https://www.gnu.org/licenses/agpl-3.0.html",
        label_key: "footer.licensing",
        is_external: true,
    },
    FooterLink {
        href: "mailto:info@xodium.org",
        label_key: "footer.contact",
        is_external: true,
    },
];

const GIT_SHA: &str = env!("GIT_SHA");
const GITHUB_REPO_URL: &str = "https://github.com/XodiumSoftware/xodium.org/commit";

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
            <div class="mx-auto max-w-7xl flex flex-col-reverse md:flex-row md:justify-between items-center gap-4">
                <aside class="max-md:w-full max-md:text-center" style="container-type: inline-size">
                    <div class="flex flex-col md:flex-row md:items-center gap-1 md:gap-0">
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
                                class="link link-hover text-base-content/50 hover:link-primary font-mono text-sm md:ml-1"
                            >
                                {format!("#{GIT_SHA}")}
                            </a>
                        })}
                    </div>
                </aside>
                <nav class="flex flex-row gap-4">
                    {FOOTER_LINKS
                        .iter()
                        .map(|link| {
                            let label = match link.label_key {
                                "footer.about" => t_string!(i18n, footer.about),
                                "footer.licensing" => t_string!(i18n, footer.licensing),
                                "footer.contact" => t_string!(i18n, footer.contact),
                                _ => link.label_key,
                            };
                            let target = if link.is_external { Some("_blank") } else { None };
                            let rel = if link.is_external { Some("noopener noreferrer") } else { None };
                            view! {
                                <a
                                    href=link.href
                                    target=target
                                    rel=rel
                                    class="link link-hover link-primary"
                                >
                                    {label}
                                </a>
                            }
                        })
                        .collect::<Vec<_>>()}
                </nav>
            </div>
        </footer>
    }
}
