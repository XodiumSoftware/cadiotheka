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
    maxlength: usize,
    #[prop(into, default = "min-h-[8rem]".to_string())] editor_class: String,
) -> impl IntoView {
    let (tab, set_tab) = signal(MarkdownEditorTab::Write);

    let (history, set_history) = signal(Vec::<String>::new());
    let (history_index, set_history_index) = signal(0usize);
    let history_read: Signal<Vec<String>> = history.into();
    let history_index_read: Signal<usize> = history_index.into();
    let (last_selection, set_last_selection) = signal((0usize, 0usize));
    let textarea_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();

    let push_history = {
        let history_index = history_index_read;
        let history = history_read;
        move |next: String| {
            let next = next.replace('\r', "");
            set_history.update(|history| {
                let idx = history_index.get_untracked();
                if history.len() > idx + 1 {
                    history.truncate(idx + 1);
                }
                if history.last() != Some(&next) {
                    history.push(next.clone());
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }
            });
            let new_idx = history_index
                .get_untracked()
                .saturating_add(1)
                .min(history.get_untracked().len().saturating_sub(1));
            set_history_index.set(new_idx);
        }
    };

    Effect::new(move |_| {
        let initial = value.get();
        if !initial.is_empty() && history_read.get_untracked().is_empty() {
            set_history.set(vec![initial.replace('\r', "")]);
            set_history_index.set(0);
        }
    });

    let with_selection = move |f: &SelectionTransform| {
        let current = value.get_untracked();
        let (start, end) = last_selection.get_untracked();
        let (next, new_start, new_end) =
            f(&current, start.min(current.len()), end.min(current.len()));
        let next = next.replace('\r', "");
        push_history(next.clone());
        on_input.run(next.clone());
        set_last_selection.set((new_start.min(next.len()), new_end.min(next.len())));
        if let Some(textarea) = textarea_ref.get() {
            let _ = textarea.set_selection_start(Some(new_start as u32));
            let _ = textarea.set_selection_end(Some(new_end as u32));
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
                    <ToolbarButton label="Bold" on_click=Callback::new(move |_| apply_wrap("**", "**"))>
                        "B"
                    </ToolbarButton>
                    <ToolbarButton label="Italic" on_click=Callback::new(move |_| apply_wrap("*", "*"))>
                        "I"
                    </ToolbarButton>
                    <ToolbarButton label="Heading" on_click=Callback::new(move |_| insert_snippet("## Heading"))>
                        "H"
                    </ToolbarButton>
                    <ToolbarButton label="Code" on_click=Callback::new(move |_| apply_wrap("`", "`"))>
                        "<>"
                    </ToolbarButton>
                    <ToolbarButton label="Link" on_click=Callback::new(move |_| insert_snippet("[label](https://example.com)"))>
                        "🔗"
                    </ToolbarButton>
                    <ToolbarButton label="Bullet list" on_click=Callback::new(move |_| apply_line_prefix("- "))>
                        "•"
                    </ToolbarButton>
                    <ToolbarButton label="Numbered list" on_click=Callback::new(move |_| insert_snippet("1. Item\n2. Item"))>
                        "1."
                    </ToolbarButton>
                    <ToolbarButton label="Task list" on_click=Callback::new(move |_| insert_snippet("- [ ] Task"))>
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
                                class=format!("textarea w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none {}", editor_class)
                                maxlength=maxlength.to_string()
                                prop:value=value.get()
                                on:input=move |ev| {
                                    let next = event_target_value(&ev).replace('\r', "");
                                    if let Some(textarea) = event_target_textarea(&ev) {
                                        set_last_selection.set((
                                            textarea.selection_start().unwrap_or_default().unwrap_or(0) as usize,
                                            textarea.selection_end().unwrap_or_default().unwrap_or(0) as usize,
                                        ));
                                    }
                                    push_history(next.clone());
                                    on_input.run(next);
                                }
                                on:keydown=move |ev| {
                                    if ev.key() == "Tab" {
                                        ev.prevent_default();
                                        let current = value.get_untracked();
                                        let next = format!("{current}    ").replace('\r', "");
                                        push_history(next.clone());
                                        on_input.run(next);
                                    }
                                    let ctrl = ev.ctrl_key() || ev.meta_key();
                                    if ctrl && ev.key().eq_ignore_ascii_case("z") {
                                        ev.prevent_default();
                                        let idx = history_index.get_untracked();
                                        if idx > 0 {
                                            let current_len = value.get_untracked().len();
                                            let target = history.get_untracked()[idx - 1].clone();
                                            let target_len = target.len();
                                            let (start, end) = last_selection.get_untracked();
                                            let new_start = start.min(target_len);
                                            let new_end = if end == current_len || end > target_len {
                                                target_len
                                            } else {
                                                end.min(target_len)
                                            };
                                            set_last_selection.set((new_start, new_end));
                                            set_history_index.set(idx - 1);
                                            on_input.run(target);
                                            if let Some(textarea) = textarea_ref.get() {
                                                let _ = textarea.set_selection_start(Some(new_start as u32));
                                                let _ = textarea.set_selection_end(Some(new_end as u32));
                                            }
                                        }
                                    }
                                    if ctrl
                                        && (ev.key().eq_ignore_ascii_case("y")
                                            || (ev.shift_key()
                                                && ev.key().eq_ignore_ascii_case("z")))
                                    {
                                        ev.prevent_default();
                                        let idx = history_index.get_untracked();
                                        let history_len = history.get_untracked().len();
                                        if idx + 1 < history_len {
                                            let current_len = value.get_untracked().len();
                                            let target = history.get_untracked()[idx + 1].clone();
                                            let target_len = target.len();
                                            let (start, end) = last_selection.get_untracked();
                                            let new_start = start.min(target_len);
                                            let new_end = if end == current_len || end > target_len {
                                                target_len
                                            } else {
                                                end.min(target_len)
                                            };
                                            set_last_selection.set((new_start, new_end));
                                            set_history_index.set(idx + 1);
                                            on_input.run(target);
                                            if let Some(textarea) = textarea_ref.get() {
                                                let _ = textarea.set_selection_start(Some(new_start as u32));
                                                let _ = textarea.set_selection_end(Some(new_end as u32));
                                            }
                                        }
                                    }
                                }
                                autofocus
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
                    if on_cancel.is_some() && on_save.is_some() {
                        view! {
                            <div class="flex items-center justify-between gap-2">
                                <span class="text-xs text-base-content/50">
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
                                <span class="text-xs text-base-content/50">
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
            class="btn btn-ghost btn-xs min-h-0 h-7 px-2"
            title=label
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
