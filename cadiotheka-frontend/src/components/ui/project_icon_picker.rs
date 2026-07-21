use crate::data::IconUrl;
use crate::utils::{placeholder_color, placeholder_letter};
use leptos::prelude::*;

/// A clickable project icon preview that toggles an edit state.
///
/// Displays the provided icon URL or a colored placeholder derived from the
/// title. When `editable` is true, hovering shows a pencil overlay and clicking
/// invokes `on_click`. The caller controls the dimensions via `class`.
#[component]
pub fn ProjectIconPicker(
    #[prop(into)] icon_url: Signal<Option<IconUrl>>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] editable: Signal<bool>,
    #[prop(into)] on_click: Callback<()>,
    #[prop(into, default = "w-16 h-16".to_string())] class: String,
) -> impl IntoView {
    let url = Memo::new(move |_| {
        icon_url
            .get()
            .as_ref()
            .map(|IconUrl(url)| url.clone())
            .unwrap_or_default()
    });
    let letter = Memo::new(move |_| placeholder_letter(&title.get()));
    let bg = Memo::new(move |_| placeholder_color(&title.get()));

    view! {
        <button
            type="button"
            class=format!(
                "group relative rounded overflow-hidden focus:outline-none focus:ring-2 focus:ring-primary {}",
                class
            )
            aria-label="Edit project icon"
            disabled=move || !editable.get()
            on:click=move |_| on_click.run(())
        >
            {move || {
                let url = url.get();
                if url.trim().is_empty() {
                    view! {
                        <div
                            class=format!("w-full h-full flex items-center justify-center text-white font-bold text-xl {}", bg.get())
                            aria-hidden="true"
                        >
                            {letter.get()}
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <img
                            src=url
                            alt=move || format!("{} icon", title.get())
                            class="w-full h-full object-cover"
                        />
                    }
                        .into_any()
                }
            }}
            {move || editable.get().then(|| view! {
                <div class="absolute inset-0 flex items-center justify-center bg-black/0 group-hover:bg-black/30 transition-colors">
                    <svg
                        class="w-5 h-5 text-white opacity-0 group-hover:opacity-100 transition-opacity"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                    </svg>
                </div>
            })}
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_uses_placeholder_for_empty_url() {
        let _picker = view! {
            <ProjectIconPicker
                icon_url=Signal::derive(move || None)
                title=Signal::derive(move || "My Project".to_string())
                editable=Signal::derive(move || false)
                on_click=Callback::new(|_| {})
                class="w-16 h-16"
            />
        };
    }

    #[test]
    fn picker_uses_image_for_present_url() {
        let _picker = view! {
            <ProjectIconPicker
                icon_url=Signal::derive(move || Some(IconUrl("https://example.com/icon.svg".to_string())))
                title=Signal::derive(move || "My Project".to_string())
                editable=Signal::derive(move || true)
                on_click=Callback::new(|_| {})
                class="w-16 h-16"
            />
        };
    }
}
