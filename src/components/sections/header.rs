use crate::components::ui::toggle::ToggleSliderWithSlashLabel;
use crate::context::LayoutContext;
use crate::i18n::{t_string, use_i18n};
use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;
use std::time::Duration;

#[component]
pub fn Header() -> impl IntoView {
    let i18n = use_i18n();
    let layout = LayoutContext::use_context();
    let (is_scrolled, set_is_scrolled) = signal(false);
    let (is_logo_active, set_is_logo_active) = signal(false);
    let (letters_visible, set_letters_visible) = signal(true);

    // Scroll listener for backdrop blur
    Effect::new(move |_| {
        window_event_listener::<web_sys::Event, _>("scroll", move |_ev| {
            let scrolled = web_sys::window()
                .map(|w| w.scroll_y().unwrap_or(0.0) > 0.0)
                .unwrap_or(false);
            set_is_scrolled.set(scrolled);
        });
    });

    let trigger_logo_animation = move || {
        if is_logo_active.get() {
            return;
        }
        set_letters_visible.set(false);
        set_is_logo_active.set(true);
        spawn_local(async move {
            gloo_timers::future::sleep(Duration::from_millis(1500)).await;
            set_is_logo_active.set(false);
            gloo_timers::future::sleep(Duration::from_millis(300)).await;
            set_letters_visible.set(true);
        });
    };

    let logo_wordmark = t_string!(i18n, header.logo_wordmark);
    let letter_elements = logo_wordmark
        .chars()
        .enumerate()
        .map(|(idx, ch)| {
            let delay = format!("{}ms", idx * 25);
            let ch = ch.to_string();
            view! {
                <span
                    class=move || {
                        if letters_visible.get() {
                            "inline-block transition-all duration-300 ease-out opacity-100 translate-x-0".to_string()
                        } else {
                            "inline-block transition-all duration-300 ease-in opacity-0 -translate-x-8 scale-0".to_string()
                        }
                    }
                    style=format!("transition-delay: {}", delay)
                >
                    {ch}
                </span>
            }
        })
        .collect::<Vec<_>>();

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
                <div class="navbar-start gap-8">
                    <a
                        href="#"
                        class="flex items-center gap-4 p-0 group"
                        on:click=move |ev: leptos::web_sys::MouseEvent| {
                            ev.prevent_default();
                            if let Some(window) = web_sys::window() {
                                window.scroll_to_with_x_and_y(0.0, 0.0);
                            }
                            trigger_logo_animation();
                        }
                    >
                        <span
                            class=move || {
                                if is_logo_active.get() {
                                    "logo-container logo-active inline-block".to_string()
                                } else {
                                    "logo-container inline-block".to_string()
                                }
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
                        </span>
                        <span class="text-2xl font-bold tracking-tight text-base-content group-hover:text-primary transition-colors overflow-hidden whitespace-nowrap">
                            {letter_elements.into_iter().collect_view()}
                        </span>
                    </a>
                </div>

                <div class="navbar-end">
                    <ToggleSliderWithSlashLabel
                        checked=layout.wide
                        on_change=move |value| layout.set_wide.set(value)
                        label_left=t_string!(i18n, projects.narrow_mode)
                        label_right=t_string!(i18n, projects.wide_mode)
                        shortcut_hint="Alt + L"
                    />
                </div>
            </nav>
        </header>
    }
}
