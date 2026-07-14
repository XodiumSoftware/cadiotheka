use crate::components::cards::teamcard::{TeamCard, TeamCardProperties};
use crate::components::ui::cornerframe::CornerFrame;
use crate::components::ui::datagrid::data_grid;
use crate::github::{Member, fetch_members};
use crate::i18n::{t, t_string, use_i18n};
use crate::utils::{observe_intersections, window_event_listener};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

#[component]
pub fn TeamDeckSection() -> impl IntoView {
    let i18n = use_i18n();
    let (rotation, set_rotation) = signal(0usize);
    let (count, set_count) = signal(8usize);
    let (retry_count, set_retry_count) = signal(0u32);
    let (is_visible, set_is_visible) = signal(false);
    let (should_focus_front, set_should_focus_front) = signal(false);

    let resource = LocalResource::new(move || {
        let _ = retry_count.get();
        async move { fetch_members().await }
    });

    let retry = move || {
        set_retry_count.update(|n| *n += 1);
    };

    let rotate = move || {
        let total = count.get();
        if total > 0 {
            set_rotation.set((rotation.get() + total - 1) % total);
            set_should_focus_front.set(true);
        }
    };

    // Track when the team section is visible so keyboard shortcuts work on scroll
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(section) = document.get_element_by_id("team") else {
            return;
        };

        observe_intersections(&[section], 0.5, move |entries| {
            for entry in entries {
                if entry.is_intersecting() {
                    set_is_visible.set(true);
                } else {
                    set_is_visible.set(false);
                    set_rotation.set(0);
                }
            }
        });
    });

    // Move focus to the front card after each rotation
    Effect::new(move |_| {
        if !should_focus_front.get() {
            return;
        }
        set_should_focus_front.set(false);

        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };

        let total = count.get();
        let rot = rotation.get();
        if total == 0 {
            return;
        }

        // The title card is front when rotation % total == 0.
        // Member card with 1-based index i is front when (i + rot) % total == 0.
        let front_id = if rot % total == 0 {
            "team-card-title".to_string()
        } else {
            let front_idx = (total - rot) % total;
            format!("team-card-{front_idx}")
        };

        if let Some(el) = document.get_element_by_id(&front_id) {
            let _ = el.dyn_into::<web_sys::HtmlElement>().map(|h| h.focus());
        }
    });

    // Global keydown listener that fires when the team section is in view
    Effect::new(move |_| {
        let rotate = rotate;
        let is_visible = is_visible;

        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if !is_visible.get() {
                return;
            }

            let key = ev.key();
            if key == " " {
                ev.prevent_default();
                rotate();
            } else if key == "Enter" {
                ev.prevent_default();
                let _ = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                    .map(|el| el.click());
            }
        });
    });

    view! {
        <section id="team" class="relative py-24 sm:py-32">
            <div class="team-deck-bg" />
            <div class="relative z-10 mx-auto max-w-7xl px-6 lg:px-8">
                <div
                    class="team-deck-wrapper"
                    tabindex="0"
                    role="region"
                    aria-label=t_string!(i18n, team.deck_label)
                >
                    // Click zone at the right edge - triggers rotation
                    <div
                        class="deck-hover-zone"
                        role="button"
                        aria-label=t_string!(i18n, team.rotate_label)
                        on:click=move |_| rotate()
                    />
                    <ul class="team-deck">
                        {data_grid(
                            resource,
                            move || t!(i18n, team.empty),
                            move |members: Vec<Member>| {
                                let member_count = members.len();
                                let total_count = member_count + 1;
                                set_count.set(total_count.max(2));

                                members
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, member)| {
                                        let card_idx = idx + 1;
                                        let login = member.login.clone();
                                        let profile_url = member.html_url.clone();
                                        view! {
                                            <li
                                                id=format!("team-card-{card_idx}")
                                                class="team-deck-card"
                                                tabindex="0"
                                                role="button"
                                                aria-label=format!("{login} - {}", t_string!(i18n, team.rotate_label))
                                                data-deck-pos=move || {
                                                    let total = count.get();
                                                    (card_idx + rotation.get()) % total
                                                }
                                                on:click=move |_| {
                                                    let _ = web_sys::window().map(|w| w.open_with_url(&profile_url));
                                                }
                                            >
                                                <TeamCard props=TeamCardProperties {
                                                    login: member.login,
                                                    html_url: member.html_url,
                                                    avatar_url: member.avatar_url,
                                                    role: member.role,
                                                } />
                                            </li>
                                        }
                                    })
                                    .collect_view()
                            },
                            Some(retry),
                            t_string!(i18n, team.retry),
                        )}
                        // Title card (always present, rotates through positions)
                        <li
                            id="team-card-title"
                            class="team-deck-card team-deck-title"
                            tabindex="0"
                            role="button"
                            aria-label=t_string!(i18n, team.rotate_label)
                            data-deck-pos=move || {
                                let total = count.get();
                                rotation.get() % total
                            }
                            on:click=move |_| rotate()
                        >
                            <div class="h-full w-full p-2">
                                <CornerFrame
                                    style="square"
                                    black=true
                                    class="h-full w-full flex items-center justify-center"
                                >
                                    <h2 class="text-3xl font-bold tracking-tight text-transparent bg-base-100 bg-clip-text whitespace-nowrap">
                                        {t!(i18n, team.title)}
                                    </h2>
                                </CornerFrame>
                            </div>
                        </li>
                    </ul>
                </div>
                // Keyboard hint below the deck
                <div class="mt-6 hidden md:flex items-center justify-center gap-2 text-sm text-base-content/50 font-mono select-none">
                    <span>{t!(i18n, team.keyboard_hint_press)}</span>
                    <kbd class="inline-flex items-center justify-center px-2 py-0.5 min-w-[1.5rem] rounded border border-base-content/20 bg-base-200 shadow-kbd text-xs font-sans">{t!(i18n, team.keyboard_hint_space)}</kbd>
                    <span>{t!(i18n, team.keyboard_hint_to_rotate)}</span>
                    <span class="mx-1" aria-hidden="true">"·"</span>
                    <kbd class="inline-flex items-center justify-center px-2 py-0.5 min-w-[1.5rem] rounded border border-base-content/20 bg-base-200 shadow-kbd text-xs font-sans">{t!(i18n, team.keyboard_hint_enter)}</kbd>
                    <span>{t!(i18n, team.keyboard_hint_enter_to_open)}</span>
                </div>
            </div>
        </section>
    }
}
