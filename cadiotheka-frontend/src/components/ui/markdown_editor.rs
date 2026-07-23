use crate::components::ui::markdown::MarkdownView;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkdownEditorTab {
    Write,
    Preview,
}

type SelectionTransform = dyn Fn(&str, usize, usize) -> (String, usize, usize);

#[component]
pub fn MarkdownEditor(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into, optional)] on_cancel: Option<Callback<()>>,
    #[prop(into, optional)] on_save: Option<Callback<()>>,
    #[prop(into, default = false)] hide_actions: bool,
    maxlength: usize,
    #[prop(into, default = "min-h-[8rem]".to_string())] editor_class: String,
) -> impl IntoView {
    let (tab, set_tab) = signal(MarkdownEditorTab::Write);
    let (last_selection, set_last_selection) = signal((0usize, 0usize));
    let textarea_class = Signal::derive({
        let editor_class = editor_class.clone();
        move || {
            let at_max = value.get().len() >= maxlength;
            format!(
                "textarea w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none {} {}",
                editor_class,
                if at_max { "hover:border-error" } else { "" }
            )
        }
    });
    let textarea_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let initial_value = value.get_untracked();
    Effect::new(move |_| {
        if let Some(textarea) = textarea_ref.get() {
            textarea.set_value(&initial_value);
        }
    });

    let with_selection = move |f: &SelectionTransform| {
        let current = value.get_untracked();
        let (start, end) = last_selection.get_untracked();
        let (next, new_start, new_end) =
            f(&current, start.min(current.len()), end.min(current.len()));
        let next = next.replace('\r', "");
        on_input.run(next.clone());
        set_last_selection.set((new_start.min(next.len()), new_end.min(next.len())));
        if let Some(textarea) = textarea_ref.get() {
            let _ = textarea.set_selection_start(Some(u32::try_from(new_start).unwrap_or(0)));
            let _ = textarea.set_selection_end(Some(u32::try_from(new_end).unwrap_or(0)));
        }
    };

    let apply_wrap = move |prefix: &'static str, suffix: &'static str| {
        with_selection(&move |text: &str, start, end| {
            let before = &text[..start];
            let selected = &text[start..end];
            let after = &text[end..];
            let wrapped = format!("{before}{prefix}{selected}{suffix}{after}");
            let new_start = start + prefix.len();
            let new_end = new_start + selected.len();
            (wrapped, new_start, new_end)
        });
    };

    let apply_line_prefix = move |prefix: &'static str| {
        with_selection(&move |text: &str, start, end| {
            let before = &text[..start];
            let selected = &text[start..end];
            let after = &text[end..];
            let wrapped_selected = selected
                .lines()
                .map(|line| {
                    if line.is_empty() {
                        prefix.trim_end().to_string()
                    } else {
                        format!("{prefix}{line}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let wrapped = format!("{before}{wrapped_selected}{after}");
            let added_per_line = prefix.len();
            let lines_count = selected.lines().count().max(1);
            let new_start = start + added_per_line;
            let new_end = end + added_per_line * lines_count;
            (wrapped, new_start, new_end)
        });
    };

    let insert_snippet = move |snippet: &'static str| {
        with_selection(&move |text: &str, start, end| {
            let before = &text[..start];
            let after = &text[end..];
            let separator = if before.is_empty() || before.ends_with('\n') {
                ""
            } else {
                "\n"
            };
            let inserted = format!("{before}{separator}{snippet}{after}");
            let insert_start = before.len() + separator.len();
            let insert_end = insert_start + snippet.len();
            (inserted, insert_start, insert_end)
        });
    };

    view! {
        <div class="rounded-none border border-base-content/20 bg-base-200/30 overflow-hidden">
            <div class="flex items-center justify-between border-b border-base-content/10 px-3 pt-2 gap-3 flex-wrap">
                <div class="tabs tabs-border">
                    <button
                        type="button"
                        class=move || {
                            if tab.get() == MarkdownEditorTab::Write {
                                "tab tab-active"
                            } else {
                                "tab"
                            }
                        }
                        on:click=move |_| set_tab.set(MarkdownEditorTab::Write)
                    >
                        "Write"
                    </button>
                    <button
                        type="button"
                        class=move || {
                            if tab.get() == MarkdownEditorTab::Preview {
                                "tab tab-active"
                            } else {
                                "tab"
                            }
                        }
                        on:click=move |_| set_tab.set(MarkdownEditorTab::Preview)
                    >
                        "Preview"
                    </button>
                </div>

                <div class="flex items-center gap-1 flex-wrap text-base-content/70">
                    <ToolbarButton label="Bold" on_click=Callback::new(move |()| apply_wrap("**", "**"))>
                        "B"
                    </ToolbarButton>
                    <ToolbarButton label="Italic" on_click=Callback::new(move |()| apply_wrap("*", "*"))>
                        "I"
                    </ToolbarButton>
                    <ToolbarButton label="Heading" on_click=Callback::new(move |()| insert_snippet("## Heading"))>
                        "H"
                    </ToolbarButton>
                    <ToolbarButton label="Code" on_click=Callback::new(move |()| apply_wrap("`", "`"))>
                        "<>"
                    </ToolbarButton>
                    <ToolbarButton label="Link" on_click=Callback::new(move |()| insert_snippet("[label](https://example.com)"))>
                        "🔗"
                    </ToolbarButton>
                    <ToolbarButton label="Bullet list" on_click=Callback::new(move |()| apply_line_prefix("- "))>
                        "•"
                    </ToolbarButton>
                    <ToolbarButton label="Numbered list" on_click=Callback::new(move |()| insert_snippet("1. Item\n2. Item"))>
                        "1."
                    </ToolbarButton>
                    <ToolbarButton label="Task list" on_click=Callback::new(move |()| insert_snippet("- [ ] Task"))>
                        "☑"
                    </ToolbarButton>
                </div>
            </div>

            <div class="p-3 space-y-3">
                {move || {
                    if tab.get() == MarkdownEditorTab::Write {
                        view! {
                            <textarea
                                node_ref=textarea_ref
                                class=textarea_class
                                maxlength=maxlength.to_string()
                                on:input=move |ev| {
                                    let next = event_target_value(&ev).replace('\r', "");
                                    if let Some(textarea) = event_target_textarea(&ev) {
                                        set_last_selection.set((
                                            textarea.selection_start().unwrap_or_default().unwrap_or(0) as usize,
                                            textarea.selection_end().unwrap_or_default().unwrap_or(0) as usize,
                                        ));
                                    }
                                    on_input.run(next);
                                }
                                on:keydown=move |ev| {
                                    if ev.key() == "Tab" {
                                        ev.prevent_default();
                                        let current = value.get_untracked();
                                        let next = format!("{current}    ").replace('\r', "");
                                        on_input.run(next);
                                    }
                                }
                            ></textarea>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class=format!("min-h-[8rem] {}", editor_class)>
                                <MarkdownView source=value.get() />
                            </div>
                        }
                            .into_any()
                    }
                }}

                {move || {
                    if !hide_actions && on_cancel.is_some() && on_save.is_some() {
                        view! {
                            <div class="flex items-center justify-between gap-2">
                                <span class=move || {
                                    if value.get().len() >= maxlength {
                                        "text-xs text-error"
                                    } else {
                                        "text-xs text-base-content/50"
                                    }
                                }>
                                    {move || format!("{}/{}", value.get().len(), maxlength)}
                                </span>
                                <div class="flex gap-2">
                                    <button
                                        type="button"
                                        class="btn btn-ghost btn-xs"
                                        on:click=move |_| {
                                            if let Some(cb) = on_cancel {
                                                cb.run(());
                                            }
                                        }
                                    >"Cancel"</button>
                                    <button
                                        type="button"
                                        class="btn btn-primary btn-xs"
                                        on:click=move |_| {
                                            if let Some(cb) = on_save {
                                                cb.run(());
                                            }
                                        }
                                    >"Save"</button>
                                </div>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="flex items-center justify-between gap-2">
                                <span class=move || {
                                    if value.get().len() >= maxlength {
                                        "text-xs text-error"
                                    } else {
                                        "text-xs text-base-content/50"
                                    }
                                }>
                                    {move || format!("{}/{}", value.get().len(), maxlength)}
                                </span>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn ToolbarButton(
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="btn btn-ghost btn-xs min-h-0 h-7 px-2 tooltip tooltip-top"
            data-tip=label
            aria-label=label
            on:click=move |_| on_click.run(())
        >
            {children()}
        </button>
    }
}

fn event_target_textarea(
    ev: &leptos::web_sys::Event,
) -> Option<leptos::web_sys::HtmlTextAreaElement> {
    ev.target()
        .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlTextAreaElement>().ok())
}

fn event_target_value(ev: &leptos::web_sys::Event) -> String {
    event_target_textarea(ev)
        .map(|textarea| textarea.value())
        .unwrap_or_default()
}
