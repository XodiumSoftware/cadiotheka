use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;
use std::time::Duration;

#[component]
pub fn Header() -> impl IntoView {
    let (is_scrolled, set_is_scrolled) = signal(false);
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
                <div class="navbar-start">
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
                </div>
            </nav>
        </header>
    }
}
