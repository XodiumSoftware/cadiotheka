use crate::components::ui::modals::search::SearchModal;
use crate::contexts::{
    AddProjectModalContext, CurrentUserContext, LayoutContext, LoginModalContext,
    ProfileModalContext, ProjectsContext, SearchContext,
};
use crate::engines::{SearchEngine, Suggestion, SuggestionKind};
use crate::utils::{
    auth_url, encode_redirect_url, placeholder_color, placeholder_letter, window_event_listener,
};
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

/// Groups suggestions by kind, limiting each group to 8 items.
fn group_and_filter_suggestions(suggestions: &[Suggestion]) -> Vec<SuggestionGroup> {
    let max_per_group = 8;

    let filter_kind = |kind: SuggestionKind| {
        suggestions
            .iter()
            .filter(|s| s.kind == kind)
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

/// Applies a clicked suggestion to a query, replacing the partial token when
/// completing a prefixed suggestion.
fn apply_suggestion(query: &str, text: &str, kind: SuggestionKind) -> String {
    let mut parts: Vec<String> = query
        .split_whitespace()
        .map(std::borrow::ToOwned::to_owned)
        .collect();

    match kind {
        SuggestionKind::Sort => {
            parts.retain(|p| !p.starts_with("@sort:"));
            if parts.last().is_some_and(|p| p.starts_with('@')) {
                parts.pop();
            }
            parts.push(text.to_owned());
        }
        SuggestionKind::Author => {
            let replacement = format!("@author:{text}");
            replace_last_token(&mut parts, replacement);
        }
        SuggestionKind::Filter => {
            let replacement = format!("#{text}");
            replace_last_token(&mut parts, replacement);
        }
        SuggestionKind::Plain => {
            let needs_space = !query.is_empty() && !query.ends_with(' ');
            if needs_space {
                return format!("{query} {text}");
            }
            return format!("{query}{text}");
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
    const ACCOUNT_MENU_ITEMS: usize = 4;

    let _layout = LayoutContext::use_context();
    let search = SearchContext::use_context();
    let (is_scrolled, set_is_scrolled) = signal(false);
    let (search_open, set_search_open) = signal(false);
    let (account_menu_open, set_account_menu_open) = signal(false);
    let account_menu_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let input_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let (selected_index, set_selected_index) = signal::<Option<usize>>(None);
    let (keyboard_index, set_keyboard_index) = signal::<Option<usize>>(None);
    let avatar_button_ref: NodeRef<leptos::html::Button> = NodeRef::new();
    let profile_ref: NodeRef<leptos::html::Button> = NodeRef::new();
    let my_projects_ref: NodeRef<leptos::html::Button> = NodeRef::new();
    let my_favorites_ref: NodeRef<leptos::html::Button> = NodeRef::new();
    let logout_ref: NodeRef<leptos::html::Button> = NodeRef::new();
    let (active_menu_index, set_active_menu_index) = signal(0usize);

    let projects_ctx = ProjectsContext::use_context();

    let suggestions = Memo::new(move |_| {
        let query = search.query.get();
        let projects = projects_ctx.projects.get();
        let engine = SearchEngine::new(projects);
        let all = engine.suggestions(&query);
        group_and_filter_suggestions(&all)
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

    let current_user_ctx = CurrentUserContext::use_context();
    let login_modal_ctx = LoginModalContext::use_context();
    let profile_modal_ctx = ProfileModalContext::use_context();

    let focus_avatar = Callback::new(move |()| {
        if let Some(btn) = avatar_button_ref.get() {
            let _ = btn.focus();
        }
    });

    let activate_menu_item = Callback::new(move |idx: usize| {
        set_account_menu_open.set(false);
        let Some(account) = current_user_ctx.account.get_untracked() else {
            return;
        };
        match idx {
            0 => profile_modal_ctx.open(account),
            1 => {
                let author_query = format!("@author:{}", account.username);
                search.set_query.set(author_query);
            }
            2 => {
                let favorites_query = format!("@favorited_by:{}", account.id);
                search.set_query.set(favorites_query);
            }
            3 => {
                if let Some(window) = leptos::web_sys::window() {
                    let _ = window
                        .location()
                        .set_href(&encode_redirect_url(&auth_url("/logout")));
                }
            }
            _ => {}
        }
        if idx != 3 {
            focus_avatar.run(());
        }
    });

    Effect::new(move |_| {
        let current_user = current_user_ctx;
        let login_modal = login_modal_ctx;
        window_event_listener::<leptos::web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if ev.default_prevented() {
                return;
            }

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

            if ev.alt_key()
                && ev.key().eq_ignore_ascii_case("1")
                && current_user.account.get_untracked().is_some()
            {
                ev.prevent_default();
                activate_menu_item.run(0);
                return;
            }

            if ev.alt_key()
                && ev.key().eq_ignore_ascii_case("2")
                && current_user.account.get_untracked().is_some()
            {
                ev.prevent_default();
                activate_menu_item.run(1);
                return;
            }

            if ev.alt_key()
                && ev.key().eq_ignore_ascii_case("3")
                && current_user.account.get_untracked().is_some()
            {
                ev.prevent_default();
                activate_menu_item.run(2);
                return;
            }

            if ev.alt_key() && ev.key().eq_ignore_ascii_case("l") {
                ev.prevent_default();
                if current_user.account.get_untracked().is_some() {
                    activate_menu_item.run(3);
                } else {
                    login_modal.open();
                }
                return;
            }

            if search_open.get_untracked() && ev.key().as_str() == "Escape" {
                ev.prevent_default();
                search.set_query.set(String::new());
                set_search_open.set(false);
                set_selected_index.set(None);
                set_keyboard_index.set(None);
            }

            if account_menu_open.get_untracked() && ev.key().as_str() == "Escape" {
                ev.prevent_default();
                set_account_menu_open.set(false);
                focus_avatar.run(());
            }
        });
    });

    Effect::new(move |_| {
        let menu = account_menu_ref.get();
        let listener = window_event_listener::<web_sys::MouseEvent, _>("click", move |ev| {
            let target = ev.target();
            let menu = menu.clone();
            let should_close = target
                .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                .is_none_or(|target_node| {
                    menu.is_none_or(|menu| !menu.contains(Some(&target_node)))
                });
            if should_close {
                let was_open = account_menu_open.get_untracked();
                set_account_menu_open.set(false);
                if was_open {
                    focus_avatar.run(());
                }
            }
        });
        let _ = listener;
    });

    Effect::new(move |_| {
        window_event_listener::<web_sys::Event, _>("scroll", move |_ev| {
            let scrolled = web_sys::window().is_some_and(|w| w.scroll_y().unwrap_or(0.0) > 0.0);
            set_is_scrolled.set(scrolled);
        });
    });

    Effect::new(move |_| {
        let open = account_menu_open.get();
        let idx = active_menu_index.get();
        if !open {
            return;
        }
        spawn_local(async move {
            gloo_timers::future::sleep(Duration::from_millis(10)).await;
            let target = match idx {
                0 => profile_ref.get(),
                1 => my_projects_ref.get(),
                2 => my_favorites_ref.get(),
                3 => logout_ref.get(),
                _ => None,
            };
            if let Some(el) = target {
                let _ = el.focus();
            }
        });
    });

    Effect::new(move |_| {
        if !account_menu_open.get() {
            return;
        }
        window_event_listener::<leptos::web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if ev.default_prevented() {
                return;
            }
            let last = ACCOUNT_MENU_ITEMS.saturating_sub(1);
            match ev.key().as_str() {
                "ArrowDown" => {
                    ev.prevent_default();
                    set_active_menu_index
                        .update(|idx| *idx = if *idx >= last { 0 } else { *idx + 1 });
                }
                "ArrowUp" => {
                    ev.prevent_default();
                    set_active_menu_index
                        .update(|idx| *idx = if *idx == 0 { last } else { *idx - 1 });
                }
                "Home" => {
                    ev.prevent_default();
                    set_active_menu_index.set(0);
                }
                "End" => {
                    ev.prevent_default();
                    set_active_menu_index.set(last);
                }
                "Enter" | " " => {
                    ev.prevent_default();
                    activate_menu_item.run(active_menu_index.get_untracked());
                }
                "Tab" => {
                    set_account_menu_open.set(false);
                }
                _ => {}
            }
        });
    });

    let was_search_open = RwSignal::new(false);
    Effect::new(move |_| {
        let open = search_open.get();
        let just_opened = open && !was_search_open.get_untracked();
        was_search_open.set(open);
        if !just_opened {
            return;
        }
        let input_ref = input_ref;
        spawn_local(async move {
            gloo_timers::future::sleep(Duration::from_millis(50)).await;
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
                input.select();
            }
        });
    });

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
                    .map_or(0, |idx| (idx + 1) % flat.len());
                set_selected_index.set(Some(new_idx));
                set_keyboard_index.set(Some(new_idx));
            }
            "ArrowUp" => {
                ev.prevent_default();
                let new_idx = selected_index
                    .get_untracked()
                    .map_or(flat.len() - 1, |idx| {
                        if idx == 0 { flat.len() - 1 } else { idx - 1 }
                    });
                set_selected_index.set(Some(new_idx));
                set_keyboard_index.set(Some(new_idx));
            }
            "Enter" => {
                if let Some(idx) = selected_index.get_untracked()
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
                        }
                    >
                        <span class="inline-block">
                            <svg width="48" height="48" viewBox="0 0 128 128" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-12 w-12">
                                <g>
                                    <path d="M36.6562 73L15 94.6562V113H33.3438L55 91.3438V73H36.6562Z" stroke="url(#paint0_linear_0_1)" stroke-width="8"/>
                                </g>
                                <g>
                                    <path d="M91.3438 55L113 33.3438V15L94.6562 15L73 36.6562V55H91.3438Z" stroke="url(#paint1_linear_0_1)" stroke-width="8"/>
                                </g>
                                <g>
                                    <path d="M91.3438 73L113 94.6562V113H94.6562L73 91.3438V73H91.3438Z" stroke="url(#paint2_linear_0_1)" stroke-width="8"/>
                                </g>
                                <g>
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
                            "Cadiotheka"
                        </span>
                    </a>
                </div>

                <div class="navbar-end flex items-center gap-3" node_ref=account_menu_ref>
                    <button
                        type="button"
                        class="btn btn-primary hover:btn-warning btn-lift"
                        on:click=move |_| set_search_open.set(true)
                        aria-label="Open search"
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
                        <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 min-w-[1.25rem] rounded border border-black/30 bg-black/10 text-black shadow-kbd text-xs font-sans ml-2" aria-hidden="true">"Alt + S"</kbd>
                    </button>
                    {move || {
                        let current_user = CurrentUserContext::use_context();
                        if current_user.is_loading.get() || current_user.account.get().is_none() {
                            return None;
                        }
                        Some(view! {
                            <button
                                type="button"
                                class="btn btn-primary btn-lift hidden sm:flex items-center gap-2"
                                aria-label="Add project"
                                on:click=move |_| {
                                    AddProjectModalContext::use_context().open();
                                }
                            >
                                <svg
                                    class="w-4 h-4"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    aria-hidden="true"
                                >
                                    <path d="M12 5v14M5 12h14" />
                                </svg>
                                <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 min-w-[1.25rem] rounded border border-black/30 bg-black/10 text-black shadow-kbd text-xs font-sans" aria-hidden="true">"Alt + N"</kbd>
                            </button>
                        })
                    }}

                    {move || {
                        let current_user = CurrentUserContext::use_context();
                        let login_modal = LoginModalContext::use_context();
                        let maybe_account = current_user.account.get();
                        if current_user.is_loading.get() {
                            return view! {
                                <span class="loading loading-spinner loading-sm text-primary" aria-hidden="true"></span>
                            }
                            .into_any();
                        }
                        match maybe_account {
                            Some(account) => view! {
                                <div class="relative">
                                    <button
                                        type="button"
                                        class="btn btn-ghost btn-lift h-[42px] w-[42px] p-0 overflow-hidden hover:border-base-content/30"
                                        aria-label="Open account menu"
                                        aria-controls="account-menu"
                                        aria-expanded=move || account_menu_open.get().to_string()
                                        aria-haspopup="menu"
                                        node_ref=avatar_button_ref
                                        on:click=move |_| {
                                            let will_open = !account_menu_open.get_untracked();
                                            set_account_menu_open.set(will_open);
                                            if will_open {
                                                set_active_menu_index.set(0);
                                            }
                                        }
                                    >
                                        {move || {
                                            let username = account.username.clone();
                                            let avatar_letter = placeholder_letter(&username);
                                            let avatar_bg = placeholder_color(&username);
                                            let avatar_alt = format!("{}'s avatar", account.display_name);
                                            match account.avatar_url.clone() {
                                                Some(url) => view! {
                                                    <img
                                                        class="w-full h-full object-cover"
                                                        src=url
                                                        alt=avatar_alt
                                                        loading="lazy"
                                                    />
                                                }
                                                .into_any(),
                                                None => view! {
                                                    <div class=format!("w-full h-full flex items-center justify-center text-white font-bold text-lg {}", avatar_bg)>
                                                        {avatar_letter}
                                                    </div>
                                                }
                                                .into_any(),
                                            }
                                        }}
                                    </button>
                                    {move || {
                                        if account_menu_open.get() {
                                            Some(view! {
                                                <ul
                                                    id="account-menu"
                                                    class="absolute right-0 top-full mt-2 min-w-48 w-max max-w-xs bg-base-100 border-2 border-base-content/80 shadow-lg z-50 py-1"
                                                    role="menu"
                                                >
                                                    <li role="none">
                                                        <button
                                                            type="button"
                                                            class=move || {
                                                                let base = "w-full text-left px-4 py-2 hover:bg-base-content/10 flex items-center justify-between gap-3 whitespace-nowrap";
                                                                if active_menu_index.get() == 0 {
                                                                    format!("{base} bg-base-content/10")
                                                                } else {
                                                                    base.to_string()
                                                                }
                                                            }
                                                            role="menuitem"
                                                            tabindex="-1"
                                                            aria-keyshortcuts="Alt+1"
                                                            node_ref=profile_ref
                                                            on:mouseenter=move |_| set_active_menu_index.set(0)
                                                            on:click=move |_| activate_menu_item.run(0)
                                                        >
                                                            <span>"Profile"</span>
                                                            <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd" aria-hidden="true">"Alt + 1"</kbd>
                                                        </button>
                                                    </li>
                                                    <li role="none">
                                                        <button
                                                            type="button"
                                                            class=move || {
                                                                let base = "w-full text-left px-4 py-2 hover:bg-base-content/10 flex items-center justify-between gap-3 whitespace-nowrap";
                                                                if active_menu_index.get() == 1 {
                                                                    format!("{base} bg-base-content/10")
                                                                } else {
                                                                    base.to_string()
                                                                }
                                                            }
                                                            role="menuitem"
                                                            tabindex="-1"
                                                            aria-keyshortcuts="Alt+2"
                                                            node_ref=my_projects_ref
                                                            on:mouseenter=move |_| set_active_menu_index.set(1)
                                                            on:click=move |_| activate_menu_item.run(1)
                                                        >
                                                            <span>"My projects"</span>
                                                            <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd" aria-hidden="true">"Alt + 2"</kbd>
                                                        </button>
                                                    </li>
                                                    <li role="none">
                                                        <button
                                                            type="button"
                                                            class=move || {
                                                                let base = "w-full text-left px-4 py-2 hover:bg-base-content/10 flex items-center justify-between gap-3 whitespace-nowrap";
                                                                if active_menu_index.get() == 2 {
                                                                    format!("{base} bg-base-content/10")
                                                                } else {
                                                                    base.to_string()
                                                                }
                                                            }
                                                            role="menuitem"
                                                            tabindex="-1"
                                                            aria-keyshortcuts="Alt+3"
                                                            node_ref=my_favorites_ref
                                                            on:mouseenter=move |_| set_active_menu_index.set(2)
                                                            on:click=move |_| activate_menu_item.run(2)
                                                        >
                                                            <span>"My favorites"</span>
                                                            <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd" aria-hidden="true">"Alt + 3"</kbd>
                                                        </button>
                                                    </li>
                                                    <li role="none">
                                                        <button
                                                            type="button"
                                                            class=move || {
                                                                let base = "w-full text-left px-4 py-2 hover:bg-base-content/10 text-error font-semibold flex items-center justify-between gap-3 whitespace-nowrap";
                                                                if active_menu_index.get() == 3 {
                                                                    format!("{base} bg-base-content/10")
                                                                } else {
                                                                    base.to_string()
                                                                }
                                                            }
                                                            role="menuitem"
                                                            tabindex="-1"
                                                            aria-keyshortcuts="Alt+L"
                                                            node_ref=logout_ref
                                                            on:mouseenter=move |_| set_active_menu_index.set(3)
                                                            on:click=move |_| activate_menu_item.run(3)
                                                        >
                                                            <span>"Log out"</span>
                                                            <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd" aria-hidden="true">"Alt + L"</kbd>
                                                        </button>
                                                    </li>
                                                </ul>
                                            })
                                        } else {
                                            None
                                        }
                                    }}
                                </div>
                            }
                            .into_any(),
                            None => view! {
                                <button
                                    type="button"
                                    class="btn btn-primary btn-lift flex items-center gap-2"
                                    aria-label="Log in"
                                    on:click=move |_| login_modal.open()
                                >
                                    <svg
                                        class="w-4 h-4"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        aria-hidden="true"
                                    >
                                        <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
                                        <polyline points="10 17 15 12 10 7" />
                                        <line x1="15" y1="12" x2="3" y2="12" />
                                    </svg>
                                    <kbd class="hidden sm:inline-flex items-center justify-center px-1.5 py-0.5 min-w-[1.25rem] rounded border border-black/30 bg-black/10 text-black shadow-kbd text-xs font-sans">
                                        "Alt + L"
                                    </kbd>
                                </button>
                            }
                            .into_any(),
                        }
                    }}
                </div>
            </nav>

            <SearchModal
                open=Signal::from(search_open)
                on_close=move |()| set_search_open.set(false)
            >
                <div class="space-y-0 flex flex-col min-h-0">
                    <div class="relative">
                        <input
                            type="text"
                            class="input w-full pr-20 bg-transparent !border-0 !outline-none !ring-0 focus:!outline-none focus:!ring-0"
                            placeholder="Search projects, tags, platforms, authors..."
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
                                    "absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                                }
                            }
                            aria-label="Clear search"
                            on:click=move |_| search.set_query.set(String::new())
                        >
                            "×"
                        </button>
                    </div>

                    <hr class="border-base-content/10" />

                    <div class="max-h-72 overflow-y-auto flex-1 min-h-0">
                        <div class="sr-only" role="status">
                            {move || {
                                let groups = suggestions.get();
                                let has_suggestions = groups.iter().any(|g| !g.suggestions.is_empty());
                                if has_suggestions {
                                    "Suggestions available.".to_string()
                                } else {
                                    "No suggestions available.".to_string()
                                }
                            }}
                        </div>
                        {move || {
                            let groups = suggestions.get();
                            let selected = selected_index.get();

                            if groups.iter().all(|group| group.suggestions.is_empty()) {
                                view! {
                                    <p class="text-base-content/50 text-sm px-3 py-2" aria-live="polite" aria-atomic="true">"No suggestions available."</p>
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
                                                        SuggestionKind::Author => format!("@author:{}", suggestion.text),
                                                        SuggestionKind::Filter => format!("#{}", suggestion.text),
                                                        SuggestionKind::Sort | SuggestionKind::Plain => suggestion.text.clone(),
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

                    <div class="hidden sm:flex items-center justify-end gap-4 text-xs text-base-content/50 px-3 py-2">
                        <div class="flex items-center gap-1.5">
                            <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">"esc"</kbd>
                            <span>"to dismiss"</span>
                        </div>

                        <span class="text-base-content/30" aria-hidden="true">"|"</span>

                        <div class="flex items-center gap-1.5">
                            <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">"return"</kbd>
                            <span>"to select"</span>
                        </div>
                    </div>
                </div>
            </SearchModal>
        </header>
    }
}
