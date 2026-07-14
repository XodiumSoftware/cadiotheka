use crate::i18n::{t, use_i18n};
use crate::utils::{observe_intersections, window_event_listener};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
use std::time::Duration;

struct SocialLink {
    href: &'static str,
    label: &'static str,
    is_external: bool,
    icon_path: &'static str,
    hover_color: &'static str,
}

const SOCIAL_LINKS: &[SocialLink] = &[
    SocialLink {
        href: "https://github.com/XodiumSoftware",
        label: "Github",
        is_external: true,
        icon_path: "/icons/github.svg",
        hover_color: "group-hover:bg-primary",
    },
    SocialLink {
        href: "https://buymeacoffee.com/illyrius",
        label: "Sponsor",
        is_external: true,
        icon_path: "/icons/sponsor.svg",
        hover_color: "group-hover:bg-pink-500",
    },
];

#[component]
pub fn Header() -> impl IntoView {
    let i18n = use_i18n();
    let (is_scrolled, set_is_scrolled) = signal(false);
    let (active_section, set_active_section) = signal(String::new());
    let (is_logo_active, set_is_logo_active) = signal(false);

    // Scroll listener for backdrop blur
    Effect::new(move |_| {
        window_event_listener::<web_sys::Event, _>("scroll", move |_ev| {
            let scrolled = web_sys::window()
                .map(|w| w.scroll_y().unwrap_or(0.0) > 0.0)
                .unwrap_or(false);
            set_is_scrolled.set(scrolled);
        });
    });

    // IntersectionObserver to track which section is most visible
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };

        let elements: Vec<web_sys::Element> = ["projects", "team"]
            .iter()
            .filter_map(|id| document.get_element_by_id(id))
            .collect();

        observe_intersections(&elements, 0.25, move |entries| {
            let best = entries
                .iter()
                .filter_map(|entry| {
                    let ratio = entry.intersection_ratio();
                    if ratio <= 0.0 {
                        return None;
                    }
                    let target = entry.target();
                    let target = target.dyn_ref::<web_sys::Element>()?;
                    let id = target.id();
                    if id.is_empty() {
                        return None;
                    }
                    Some((id, ratio))
                })
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((id, _)) = best {
                set_active_section.set(id);
            }
        });
    });

    let is_active = move |id: &str| active_section.get() == id;

    view! {
        <header
            id="top"
            class=move || {
                format!(
                    "z-20 relative sticky top-0 transition-all duration-300 border-b {}",
                    if is_scrolled.get() {
                        "backdrop-blur-md bg-base-100/50 border-white/10 shadow-2xl shadow-black"
                    } else {
                        "bg-transparent border-transparent"
                    },
                )
            }
        >
            <nav class="navbar max-w-7xl mx-auto">
                // Left side
                <div class="navbar-start gap-8">
                    <a
                        href="#"
                        class=move || {
                            if is_logo_active.get() {
                                "p-0 logo-container logo-active".to_string()
                            } else {
                                "p-0 logo-container".to_string()
                            }
                        }
                        on:click=move |ev: leptos::web_sys::MouseEvent| {
                            ev.prevent_default();
                            if let Some(window) = web_sys::window() {
                                window.scroll_to_with_x_and_y(0.0, 0.0);
                            }
                            if is_logo_active.get() {
                                return;
                            }
                            set_is_logo_active.set(true);
                            spawn_local(async move {
                                gloo_timers::future::sleep(Duration::from_millis(1500)).await;
                                set_is_logo_active.set(false);
                            });
                        }
                    >
                        <svg width="48" height="48" viewBox="0 0 128 128" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-12 w-12">
                            <g class="logo-piece logo-piece-bl">
                                <path d="M36.6562 73L15 94.6562V113H33.3438L55 91.3438V73H36.6562Z" stroke="url(#paint0_linear_0_1)" stroke-width="8"/>
                            </g>
                            <g class="logo-piece logo-piece-tr">
                                <path d="M91.3438 55L113 33.3438V15L94.6562 15L73 36.6562V55H91.3438Z" stroke="url(#paint1_linear_0_1)" stroke-width="8"/>
                            </g>
                            <g class="logo-piece logo-piece-br">
                                <path d="M91.3438 73L113 94.6562V113H94.6562L73 91.3438V73H91.3438Z" stroke="url(#paint2_linear_0_1)" stroke-width="8"/>
                            </g>
                            <g class="logo-piece logo-piece-tl">
                                <path d="M36.6562 55L15 33.3437L15 15L33.3438 15L55 36.6562V55H36.6562Z" stroke="url(#paint3_linear_0_1)" stroke-width="8"/>
                            </g>
                            <defs>
                                <linearGradient id="paint0_linear_0_1" x1="35" y1="69" x2="35" y2="117" gradientUnits="userSpaceOnUse">
                                    <stop stop-color="#CB2D3E"/>
                                    <stop offset="1" stop-color="#EF473A"/>
                                </linearGradient>
                                <linearGradient id="paint1_linear_0_1" x1="93" y1="11" x2="93" y2="59" gradientUnits="userSpaceOnUse">
                                    <stop stop-color="#EF473A"/>
                                    <stop offset="1" stop-color="#CB2D3E"/>
                                </linearGradient>
                                <linearGradient id="paint2_linear_0_1" x1="93" y1="69" x2="93" y2="117" gradientUnits="userSpaceOnUse">
                                    <stop stop-color="#CB2D3E"/>
                                    <stop offset="1" stop-color="#EF473A"/>
                                </linearGradient>
                                <linearGradient id="paint3_linear_0_1" x1="35" y1="11" x2="35" y2="59" gradientUnits="userSpaceOnUse">
                                    <stop stop-color="#EF473A"/>
                                    <stop offset="1" stop-color="#CB2D3E"/>
                                </linearGradient>
                            </defs>
                        </svg>
                    </a>
                    <a
                        href="#projects"
                        class="hover:text-primary text-sm font-semibold lift"
                        aria-current=move || if is_active("projects") { Some("true") } else { None }
                    >
                        {t!(i18n, nav.projects)}
                    </a>
                    <a
                        href="#team"
                        class="hover:text-primary text-sm font-semibold lift"
                        aria-current=move || if is_active("team") { Some("true") } else { None }
                    >
                        {t!(i18n, nav.team)}
                    </a>
                </div>
                // Right side
                <div class="navbar-end">
                    <ul class="menu menu-horizontal gap-2">
                        {SOCIAL_LINKS
                            .iter()
                            .map(|link| {
                                view! {
                                    <li>
                                        <a
                                            class="group hover:bg-transparent lift rounded-none"
                                            href=link.href
                                            aria-label=link.label
                                            title=link.label
                                            target=if link.is_external { "_blank" } else { "" }
                                            rel=if link.is_external {
                                                "noopener noreferrer"
                                            } else {
                                                ""
                                            }
                                        >
                                            <span
                                                class=format!(
                                                    "nav-icon w-6 h-6 block bg-base-content {} transition-colors",
                                                    link.hover_color,
                                                )
                                                style=format!("--mask-url: url('{}')", link.icon_path)
                                                aria-hidden="true"
                                            />
                                        </a>
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </ul>
                </div>
            </nav>
        </header>
    }
}
