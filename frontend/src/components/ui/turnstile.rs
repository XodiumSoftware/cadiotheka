use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, JsValue};
use leptos::web_sys;

/// Cloudflare Turnstile sitekey used for bot protection.
const TURNSTILE_SITE_KEY: &str = "0x4AAAAAAEA7QaTfmDX0gWZ2";

/// Renders a Cloudflare Turnstile widget and exposes its response token.
///
/// The widget inserts a hidden `input[name="cf-turnstile-response"]` into
/// the DOM. The token is read on demand so it can be attached to protected
/// backend requests. The component ensures the widget is explicitly rendered
/// whenever it becomes visible, because the Turnstile script only implicitly
/// renders widgets that exist on the initial page load.
#[component]
pub fn TurnstileWidget(id: &'static str, #[prop(into)] visible: Signal<bool>) -> impl IntoView {
    Effect::new(move |_| {
        if visible.get() {
            render_if_needed(id);
        }
    });

    view! {
        <div id=id></div>
    }
}

/// Reads the current Turnstile response token from the widget with the given
/// container id.
///
/// Returns `None` when the widget has not rendered a token yet.
pub fn turnstile_response(id: &str) -> Option<String> {
    let selector = format!("#{id} input[name='cf-turnstile-response']");
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.query_selector(&selector).ok())
        .flatten()
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .filter(|value| !value.is_empty())
}

/// Best-effort reset of all Turnstile widgets on the page.
///
/// Call this after a failed submission so the user can retry with a fresh
/// single-use token.
pub fn reset_turnstile() {
    reset_turnstile_internal();
}

fn reset_turnstile_internal() {
    let function = js_sys::Function::new_no_args(
        "if (typeof window !== 'undefined' && window.turnstile && typeof window.turnstile.reset === 'function') { try { window.turnstile.reset(); } catch (e) { /* widget not ready yet */ } }",
    );
    let _ = function.call0(&JsValue::undefined());
}

/// Explicitly renders the Turnstile widget into the given container if it has
/// not already rendered.
///
/// Polls briefly if the Turnstile API has not loaded yet, so widgets inside
/// modals still render even though they are added to the DOM after page load.
fn render_if_needed(id: &str) {
    let script = format!(
        "(function() {{ \
            var id = {id:?}; \
            var sitekey = {TURNSTILE_SITE_KEY:?}; \
            var action = 'turnstile-spin-v2'; \
            var tryRender = function() {{ \
                var container = document.getElementById(id); \
                if (!container) return; \
                if (container.querySelector(\"input[name='cf-turnstile-response']\")) return; \
                if (typeof window !== 'undefined' && window.turnstile && window.turnstile.render) {{ \
                    window.turnstile.render('#' + id, {{ sitekey: sitekey, action: action }}); \
                }} else {{ \
                    setTimeout(tryRender, 100); \
                }} \
            }}; \
            tryRender(); \
        }})();",
    );
    let function = js_sys::Function::new_no_args(&script);
    let _ = function.call0(&JsValue::undefined());
}
