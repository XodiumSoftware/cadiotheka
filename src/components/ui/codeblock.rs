use crate::i18n::{t, use_i18n};
use crate::utils::{prefers_reduced_motion, window_event_listener};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
use leptos_i18n::tu_string;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const ASCII_ART: &str = r"
         ,-.--,  _,.---._                 .=-.-.                   ___
.--.-.  /=/, .',-.' , -  `.   _,..---._  /==/_ /.--.-. .-.-..-._ .='.'\
\==\ -\/=/- / /==/_,  ,  - \/==/,   -  \|==|, |/==/ -|/=/  /==/ \|==|  |
 \==\ `-' ,/ |==|   .=.     |==|   _   _\==|  ||==| ,||=| -|==|,|  / - |
  |==|,  - | |==|_ : ;=:  - |==|  .=.   |==|- ||==|- | =/  |==|  \/  , |
 /==/   ,   \|==| , '='     |==|,|   | -|==| ,||==|,  \/ - |==|- ,   _ |
/==/, .--, - \\==\ -    ,_ /|==|  '='   /==|- ||==|-   ,   /==| _ /\   |
\==\- \/=/ , / '.='. -   .' |==|-,   _`//==/. //==/ , _  .'/==/  / / , /
 `--`-'  `--`    `--`--''   `-.`.____.' `--`-` `--`..---'  `--`./  `--`
";

#[component]
pub fn CodeBlock() -> impl IntoView {
    let i18n = use_i18n();
    let company_values = [
        tu_string!(i18n, landing.value_1),
        tu_string!(i18n, landing.value_2),
        tu_string!(i18n, landing.value_3),
        tu_string!(i18n, landing.value_4),
    ];

    let (show_ascii, set_show_ascii) = signal(false);
    let (lines, set_lines) = signal(Vec::<String>::new());
    let (current_line, set_current_line) = signal(0usize);
    let (cursor_visible, set_cursor_visible) = signal(true);
    let (show_buttons, set_show_buttons) = signal(false);
    let cancelled = Arc::new(AtomicBool::new(false));

    // Synchronous check for prefers-reduced-motion before spawning effects
    let prefers_reduced_motion = prefers_reduced_motion();

    // Cursor blink + typewriter effects share one cancellation flag
    Effect::new(move |_| {
        let c_blink_cleanup = Arc::clone(&cancelled);
        let c_blink_task = Arc::clone(&cancelled);
        let c_type_cleanup = Arc::clone(&cancelled);
        let c_type_task = Arc::clone(&cancelled);

        on_cleanup(move || {
            c_blink_cleanup.store(true, Ordering::Relaxed);
            c_type_cleanup.store(true, Ordering::Relaxed);
        });

        // Cursor blink loop (skipped if reduced motion preferred)
        spawn_local(async move {
            if prefers_reduced_motion {
                set_cursor_visible.set(false);
                return;
            }
            loop {
                gloo_timers::future::sleep(Duration::from_millis(530)).await;
                if c_blink_task.load(Ordering::Relaxed) {
                    break;
                }
                set_cursor_visible.update(|v| *v = !*v);
            }
        });

        // Show ASCII art first, then type out lines
        spawn_local(async move {
            if prefers_reduced_motion {
                // Instant display — no delays, no typing animation
                set_show_ascii.set(true);
                set_lines.set(company_values.iter().map(|s| s.to_string()).collect());
                set_current_line.set(company_values.len());
                set_show_buttons.set(true);
                return;
            }
            // Initial delay
            gloo_timers::future::sleep(Duration::from_millis(500)).await;
            if c_type_task.load(Ordering::Relaxed) {
                return;
            }
            set_show_ascii.set(true);

            // Wait for ASCII to be visible
            gloo_timers::future::sleep(Duration::from_millis(800)).await;
            if c_type_task.load(Ordering::Relaxed) {
                return;
            }

            for (i, line) in company_values.iter().enumerate() {
                set_current_line.set(i);

                // Type each character
                let mut typed = String::new();
                for ch in line.chars() {
                    typed.push(ch);
                    set_lines.update(|lines| {
                        if lines.len() <= i {
                            lines.push(typed.clone());
                        } else {
                            lines[i] = typed.clone();
                        }
                    });
                    gloo_timers::future::sleep(Duration::from_millis(30)).await;
                    if c_type_task.load(Ordering::Relaxed) {
                        return;
                    }
                }

                // Pause between lines
                gloo_timers::future::sleep(Duration::from_millis(400)).await;
                if c_type_task.load(Ordering::Relaxed) {
                    return;
                }
            }

            // Show buttons after typing complete
            gloo_timers::future::sleep(Duration::from_millis(600)).await;
            if c_type_task.load(Ordering::Relaxed) {
                return;
            }
            set_show_buttons.set(true);
        });
    });

    // Global keydown shortcuts for CTA buttons
    Effect::new(move |_| {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };

        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if ev.repeat() {
                return;
            }

            // Ignore shortcuts while typing in an input, textarea, or contenteditable element
            if let Some(active) = document.active_element() {
                let tag = active.tag_name().to_lowercase();
                let is_editable = tag == "input"
                    || tag == "textarea"
                    || tag == "select"
                    || active
                        .get_attribute("contenteditable")
                        .is_some_and(|v| !v.is_empty() && v != "false");
                if is_editable {
                    return;
                }
            }

            let key = ev.key().to_lowercase();
            if key == "g" {
                ev.prevent_default();
                if let Some(el) = document.get_element_by_id("cta-get-started")
                    && let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>()
                {
                    html_el.click();
                }
            } else if key == "j" {
                ev.prevent_default();
                if let Some(el) = document.get_element_by_id("cta-join-us")
                    && let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>()
                {
                    html_el.click();
                }
            }
        });
    });

    view! {
        <div class="w-full max-w-2xl mx-auto">
            // Terminal window
            <div class="backdrop-blur-sm bg-base-300/50 rounded-lg overflow-hidden shadow-2xl border border-white/10">
                // Terminal header
                <div class="bg-base-200 px-4 py-2 flex items-center gap-2 border-b border-base-content/5">
                    <div class="flex gap-1.5">
                        <div class="w-3 h-3 rounded-full bg-error/80"></div>
                        <div class="w-3 h-3 rounded-full bg-warning/80"></div>
                        <div class="w-3 h-3 rounded-full bg-success/80"></div>
                    </div>
                    <div class="flex-1 text-center text-xs text-base-content/50 font-mono">
                        {t!(i18n, landing.terminal_title)}
                    </div>
                    <div class="w-14"></div>
                </div>

                // Terminal content
                <div class="p-4 sm:p-6 font-mono text-xs sm:text-sm" style="min-height: 280px;">
                    // ASCII Art
                    {move || {
                        if show_ascii.get() {
                            view! {
                                <div class="w-full overflow-x-auto">
                                    <pre class="text-primary/90 leading-none mb-4 text-[0.5rem] sm:text-xs whitespace-pre origin-left scale-[0.75] sm:scale-100 inline-block" aria-hidden="true">
                                        {ASCII_ART}
                                    </pre>
                                </div>
                            }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}

                    {move || {
                        let current = current_line.get();
                        let cursor = cursor_visible.get();
                        let all_lines = lines.get();
                        let menu_visible = show_buttons.get();

                        view! {
                            <>
                                {all_lines.iter().enumerate().map(|(i, line)| {
                                    let is_current = i == current;
                                    // Only show cursor on current line if menu is NOT visible
                                    let show_cursor = is_current && cursor && !menu_visible;

                                    view! {
                                        <div class="text-primary/80 leading-relaxed">
                                            <span class="text-secondary/60 mr-2">"$xodium > "</span>
                                            {line.clone()}
                                            {show_cursor.then(|| view! {
                                                <span class="inline-block w-2 h-4 bg-primary/60 ml-0.5 align-middle"></span>
                                            })}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}

                                // Current empty line being typed
                                {if all_lines.len() < company_values.len() && all_lines.len() == current {
                                    view! {
                                        <div class="text-primary/80 leading-relaxed">
                                            <span class="text-secondary/60 mr-2">"$xodium > "</span>
                                            {cursor.then(|| view! {
                                                <span class="inline-block w-2 h-4 bg-primary/60 ml-0.5 align-middle"></span>
                                            })}
                                        </div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            </>
                        }
                    }}
                </div>

                // CTA Buttons area
                <div
                    class={move || {
                        if show_buttons.get() {
                            "px-4 sm:px-6 pb-6 transition-all duration-500 opacity-100 translate-y-0"
                        } else {
                            "px-4 sm:px-6 pb-6 transition-all duration-500 opacity-0 translate-y-4"
                        }
                    }}
                >
                    <div class="border-t border-base-content/10 pt-4">
                        <div class="flex flex-col sm:flex-row gap-3 justify-center">
                            <a
                                id="cta-get-started"
                                href="https://github.com/XodiumSoftware"
                                class="btn btn-primary hover:btn-warning btn-lift"
                                target="_blank"
                                rel="noopener noreferrer"
                                accesskey="g"
                            >
                                {t!(i18n, landing.cta_get_started)}
                                <kbd class="inline-flex items-center justify-center px-1.5 py-0.5 min-w-[1.25rem] rounded border border-black/30 bg-black/10 text-black shadow-kbd text-xs font-sans ml-2" aria-hidden="true">"G"</kbd>
                            </a>
                            <a
                                id="cta-join-us"
                                href="mailto:info@xodium.org"
                                class="btn btn-outline btn-outline-ghost btn-hover-warning btn-lift"
                                accesskey="j"
                            >
                                {t!(i18n, landing.cta_join_us)}
                                <kbd class="inline-flex items-center justify-center px-1.5 py-0.5 min-w-[1.25rem] rounded border border-white/30 bg-white/10 text-white shadow-kbd text-xs font-sans ml-2" aria-hidden="true">"J"</kbd>
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
