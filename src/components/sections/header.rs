use crate::components::ui::modal::Modal;
use crate::context::{LayoutContext, SearchContext};
use crate::data::load_cards;
use crate::engines::{SearchEngine, Suggestion, SuggestionKind};
use crate::i18n::{t_string, use_i18n};
use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
use std::time::Duration;

/// A grouped slice of suggestions shown under a category header.
#[derive(Clone, PartialEq)]
struct SuggestionGroup {
    title: &'static str,
    suggestions: Vec<Suggestion>,
}

/// Filters and groups suggestions by kind, limiting each group to 8 items
/// and matching the active completion needle.
fn group_and_filter_suggestions(suggestions: &[Suggestion], needle: &str) -> Vec<SuggestionGroup> {
    let needle = needle.to_lowercase();
    let max_per_group = 8;

    let filter_kind = |kind: SuggestionKind| {
        suggestions
            .iter()
            .filter(|s| s.kind == kind && s.text.to_lowercase().contains(&needle))
            .take(max_per_group)
            .cloned()
            .collect::<Vec<_>>()
    };

    vec![
        SuggestionGroup {
            title: "name:",
            suggestions: filter_kind(SuggestionKind::Plain),
        },
        SuggestionGroup {
            title: "tag:",
            suggestions: filter_kind(SuggestionKind::Filter),
        },
        SuggestionGroup {
            title: "author:",
            suggestions: filter_kind(SuggestionKind::Author),
        },
        SuggestionGroup {
            title: "sort:",
            suggestions: filter_kind(SuggestionKind::Sort),
        },
    ]
}

/// Returns the current completion needle within the active prefix token.
fn active_needle(query: &str) -> String {
    query
        .split_whitespace()
        .last()
        .map(|token| {
            if token.starts_with('@') || token.starts_with('#') {
                token[1..].to_owned()
            } else {
                token.to_owned()
            }
        })
        .unwrap_or_default()
}

/// Applies a clicked suggestion to a query, replacing the partial token when
/// completing a prefixed suggestion.
fn apply_suggestion(query: &str, text: &str, kind: SuggestionKind) -> String {
    let mut parts: Vec<String> = query.split_whitespace().map(|s| s.to_owned()).collect();

    match kind {
        SuggestionKind::Sort => {
            parts.retain(|p| !p.starts_with("@sort:"));
            if parts.last().is_some_and(|p| p.starts_with('@')) {
                parts.pop();
            }
            parts.push(text.to_owned());
        }
        SuggestionKind::Author => {
            let replacement = format!("@author:{}", text);
            replace_last_token(&mut parts, replacement);
        }
        SuggestionKind::Filter => {
            let replacement = format!("#{}", text);
            replace_last_token(&mut parts, replacement);
        }
        SuggestionKind::Plain => {
            let needs_space = !query.is_empty() && !query.ends_with(' ');
            if needs_space {
                return format!("{} {}", query, text);
            }
            return format!("{}{}", query, text);
        }
    }

    parts.join(" ")
}

/// Replaces the last whitespace-separated token with a new string.
fn replace_last_token(parts: &mut Vec<String>, replacement: String) {
    if parts.is_empty() {
        parts.push(replacement);
    } else {
        parts.pop();
        parts.push(replacement);
    }
}

#[component]
pub fn Header() -> impl IntoView {
    let i18n = use_i18n();
    let _layout = LayoutContext::use_context();
    let search = SearchContext::use_context();
    let (is_scrolled, set_is_scrolled) = signal(false);
    let (is_logo_active, set_is_logo_active) = signal(false);
    let (letters_visible, set_letters_visible) = signal(true);
    let (search_open, set_search_open) = signal(false);
    let input_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let (selected_index, set_selected_index) = signal::<Option<usize>>(None);
    let (keyboard_index, set_keyboard_index) = signal::<Option<usize>>(None);

    let cards = load_cards();
    let engine = SearchEngine::new(cards);

    let suggestions = Memo::new(move |_| {
        let query = search.query.get();
        let all = engine.suggestions(&query);
        group_and_filter_suggestions(&all, &active_needle(&query))
    });

    let insert_suggestion = move |text: String, kind: SuggestionKind| {
        let current = search.query.get();
        let new_query = apply_suggestion(&current, &text, kind);
        search.set_query.set(new_query + " ");
        set_selected_index.set(None);
        set_keyboard_index.set(None);
        input_ref.get().map(|input| input.focus().ok());
    };

    let flattened_suggestions = move || {
        suggestions
            .get()
            .into_iter()
            .flat_map(|group| group.suggestions)
            .collect::<Vec<Suggestion>>()
    };

    // Keyboard shortcuts: Alt+S opens search, Alt+C clears it, Escape closes it.
    Effect::new(move |_| {
        window_event_listener::<leptos::web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if ev.alt_key() && ev.key().eq_ignore_ascii_case("s") {
                ev.prevent_default();
                set_search_open.set(true);
                set_selected_index.set(None);
                set_keyboard_index.set(None);
                return;
            }

            if ev.alt_key() && ev.key().eq_ignore_ascii_case("c") {
                ev.prevent_default();
                search.set_query.set(String::new());
                set_selected_index.set(None);
                set_keyboard_index.set(None);
                input_ref.get().map(|input| input.focus().ok());
                return;
            }

            if search_open.get() && ev.key().as_str() == "Escape" {
                ev.prevent_default();
                search.set_query.set(String::new());
                set_search_open.set(false);
                set_selected_index.set(None);
                set_keyboard_index.set(None);
            }
        });
    });

    // Scroll listener for backdrop blur
    Effect::new(move |_| {
        window_event_listener::<web_sys::Event, _>("scroll", move |_ev| {
            let scrolled = web_sys::window()
                .map(|w| w.scroll_y().unwrap_or(0.0) > 0.0)
                .unwrap_or(false);
            set_is_scrolled.set(scrolled);
        });
    });

    // Focus the search input whenever the modal opens.
    Effect::new(move |_| {
        if search_open.get() {
            let input_ref = input_ref;
            spawn_local(async move {
                gloo_timers::future::sleep(Duration::from_millis(50)).await;
                if let Some(input) = input_ref.get() {
                    let _ = input.focus();
                    input.select();
                }
            });
        }
    });

    // Scroll the keyboard-selected suggestion into view whenever it changes.
    Effect::new(move |_| {
        if let Some(idx) = keyboard_index.get() {
            let id = format!("search-suggestion-{idx}");
            spawn_local(async move {
                gloo_timers::future::sleep(Duration::from_millis(10)).await;
                if let Some(element) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id(&id))
                    .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                {
                    element.scroll_into_view_with_bool(false);
                }
            });
        }
    });

    let trigger_logo_animation = move || {
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

    let handle_keydown = move |ev: leptos::web_sys::KeyboardEvent| {
        let flat = flattened_suggestions();
        if flat.is_empty() {
            return;
        }

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let new_idx = selected_index
                    .get_untracked()
                    .map(|idx| (idx + 1) % flat.len())
                    .unwrap_or(0);
                set_selected_index.set(Some(new_idx));
                set_keyboard_index.set(Some(new_idx));
            }
            "ArrowUp" => {
                ev.prevent_default();
                let new_idx = selected_index
                    .get_untracked()
                    .map(|idx| if idx == 0 { flat.len() - 1 } else { idx - 1 })
                    .unwrap_or(flat.len() - 1);
                set_selected_index.set(Some(new_idx));
                set_keyboard_index.set(Some(new_idx));
            }
            "Enter" => {
                if let Some(idx) = selected_index.get()
                    && let Some(suggestion) = flat.get(idx)
                {
                    ev.prevent_default();
                    let text = suggestion.text.clone();
                    let kind = suggestion.kind;
                    insert_suggestion(text, kind);
                } else {
                    ev.prevent_default();
                    set_search_open.set(false);
                }
            }
            _ => {}
        }
    };

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
                    <button
                        type="button"
                        class="btn btn-primary hover:btn-warning btn-lift"
                        on:click=move |_| set_search_open.set(true)
                        aria-label=t_string!(i18n, search.open)
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <circle cx="11" cy="11" r="8" />
                            <path d="m21 21-4.3-4.3" />
                        </svg>
                        <kbd class="inline-flex items-center justify-center px-1.5 py-0.5 min-w-[1.25rem] rounded border border-black/30 bg-black/10 text-black shadow-kbd text-xs font-sans ml-2" aria-hidden="true">{t_string!(i18n, search.shortcut_open)}</kbd>
                    </button>
                </div>
            </nav>

            <Modal
                open=Signal::from(search_open)
                on_close=move |_| set_search_open.set(false)
                on_inner_click=move |_| {
                    input_ref.get().map(|input| input.focus().ok());
                }
            >
                <div class="space-y-0 flex flex-col min-h-0">
                    <div class="relative">
                        <input
                            type="text"
                            class="input w-full pr-20 bg-transparent !border-0 !outline-none !ring-0 focus:!outline-none focus:!ring-0"
                            placeholder=t_string!(i18n, search.placeholder)
                            prop:value=move || search.query.get()
                            role="combobox"
                            aria-expanded=move || {
                                let groups = suggestions.get();
                                groups.iter().any(|group| !group.suggestions.is_empty())
                            }
                            aria-controls="search-suggestions"
                            aria-autocomplete="list"
                            aria-activedescendant=move || {
                                selected_index.get().map(|idx| format!("search-suggestion-{idx}"))
                            }
                            on:input=move |ev| {
                                search.set_query.set(event_target_value(&ev));
                                set_selected_index.set(None);
                                set_keyboard_index.set(None);
                            }
                            on:keydown=handle_keydown
                            node_ref=input_ref
                        />
                        <button
                            type="button"
                            class=move || {
                                if search.query.get().is_empty() {
                                    "hidden"
                                } else {
                                    "absolute right-10 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                                }
                            }
                            aria-label=t_string!(i18n, search.clear)
                            on:click=move |_| search.set_query.set(String::new())
                        >
                            "×"
                        </button>
                        <button
                            type="button"
                            class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                            aria-label=t_string!(i18n, search.close)
                            on:click=move |_| {
                                search.set_query.set(String::new());
                                set_search_open.set(false);
                            }
                        >
                            "✕"
                        </button>
                    </div>

                    <hr class="border-base-content/10" />

                    <div class="max-h-72 overflow-y-auto flex-1 min-h-0">
                        {move || {
                            let groups = suggestions.get();
                            let selected = selected_index.get();

                            if groups.iter().all(|group| group.suggestions.is_empty()) {
                                view! {
                                    <p class="text-base-content/50 text-sm px-3 py-2">{t_string!(i18n, search.no_suggestions)}</p>
                                }
                                    .into_any()
                            } else {
                                let mut global_index = 0usize;
                                let group_count = groups.iter().filter(|g| !g.suggestions.is_empty()).count();
                                view! {
                                    <div
                                        id="search-suggestions"
                                        role="listbox"
                                        class="py-2"
                                    >
                                        {groups
                                            .into_iter()
                                            .filter(|group| !group.suggestions.is_empty())
                                            .enumerate()
                                            .map(|(index, group)| {
                                                let is_last = index + 1 == group_count;
                                                let group_label = group.title.trim_end_matches(':').to_owned();
                                                let group_view = group.suggestions.into_iter().map(|suggestion| {
                                                    let is_selected = selected == Some(global_index);
                                                    let text = suggestion.text.clone();
                                                    let label = match suggestion.kind {
                                                        SuggestionKind::Sort => suggestion.text.clone(),
                                                        SuggestionKind::Author => format!("@author:{}", suggestion.text),
                                                        SuggestionKind::Filter => format!("#{}", suggestion.text),
                                                        SuggestionKind::Plain => suggestion.text.clone(),
                                                    };
                                                    let kind = suggestion.kind;
                                                    let item_class = if is_selected {
                                                        "flex items-center w-full px-3 py-2 bg-base-content/10 text-base-content rounded"
                                                    } else {
                                                        "flex items-center w-full px-3 py-2 text-base-content hover:bg-base-content/5 rounded"
                                                    };
                                                    let index = global_index;
                                                    global_index += 1;
                                                    let id = format!("search-suggestion-{index}");
                                                    view! {
                                                        <button
                                                            type="button"
                                                            role="option"
                                                            aria-selected=is_selected
                                                            class=item_class
                                                            id=id
                                                            on:click=move |_| insert_suggestion(text.clone(), kind)
                                                            on:mouseenter=move |_| set_selected_index.set(Some(index))
                                                        >
                                                            <span class="truncate">{label}</span>
                                                        </button>
                                                    }
                                                }).collect_view();

                                                view! {
                                                    <div
                                                        role="group"
                                                        aria-label=group_label
                                                        class="space-y-0"
                                                    >
                                                        <div class="px-3 pt-3 pb-1 text-xs font-semibold text-base-content/40 uppercase tracking-wider" role="presentation">{group.title.trim_end_matches(':')}</div>
                                                        {group_view}
                                                        {move || {
                                                            if is_last {
                                                                None
                                                            } else {
                                                                Some(view! {
                                                                    <hr class="border-base-content/10 mt-2" />
                                                                })
                                                            }
                                                        }}
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </div>

                    <hr class="border-base-content/10" />

                    <div class="flex items-center justify-end gap-4 text-xs text-base-content/50 px-3 py-2">
                        <div class="flex items-center gap-1.5">
                            <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{t_string!(i18n, search.keyboard_esc)}</kbd>
                            <span>{t_string!(i18n, search.hint_dismiss)}</span>
                        </div>

                        <span class="text-base-content/30" aria-hidden="true">"|"</span>

                        <div class="flex items-center gap-1.5">
                            <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{t_string!(i18n, search.keyboard_return)}</kbd>
                            <span>{t_string!(i18n, search.hint_select)}</span>
                        </div>
                    </div>
                </div>
            </Modal>
        </header>
    }
}
