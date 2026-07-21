use crate::components::ui::markdown::MarkdownView;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkdownEditorTab {
    Write,
    Preview,
}

#[component]
pub fn MarkdownEditor(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
    maxlength: usize,
    #[prop(into, default = "min-h-[8rem]".to_string())] editor_class: String,
) -> impl IntoView {
    let (tab, set_tab) = signal(MarkdownEditorTab::Write);

    let apply_wrap = move |prefix: &'static str, suffix: &'static str| {
        let current = value.get_untracked();
        on_input.run(format!("{prefix}{current}{suffix}"));
    };

    let apply_line_prefix = move |prefix: &'static str| {
        let current = value.get_untracked();
        let updated = current
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
        on_input.run(updated);
    };

    let insert_snippet = move |snippet: &'static str| {
        let current = value.get_untracked();
        let separator = if current.is_empty() || current.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        on_input.run(format!("{current}{separator}{snippet}"));
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
                                class=format!("textarea w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none {}", editor_class)
                                maxlength=maxlength.to_string()
                                prop:value=value.get()
                                on:input=move |ev| on_input.run(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    if ev.key() == "Tab" {
                                        ev.prevent_default();
                                        let current = value.get_untracked();
                                        on_input.run(format!("{current}    "));
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

                <div class="flex items-center justify-between gap-2">
                    <span class="text-xs text-base-content/50">
                        {move || format!("{}/{}", value.get().len(), maxlength)}
                    </span>
                    <div class="flex gap-2">
                        <button
                            type="button"
                            class="btn btn-ghost btn-xs"
                            on:click=move |_| on_cancel.run(())
                        >"Cancel"</button>
                        <button
                            type="button"
                            class="btn btn-primary btn-xs"
                            on:click=move |_| on_save.run(())
                        >"Save"</button>
                    </div>
                </div>
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

fn event_target_value(ev: &leptos::web_sys::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlTextAreaElement>().ok())
        .map(|textarea| textarea.value())
        .unwrap_or_default()
}
