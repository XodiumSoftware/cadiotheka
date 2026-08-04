use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, JsValue};
use leptos::web_sys;

/// Renders a Cloudflare Turnstile widget and exposes its response token.
///
/// The widget inserts a hidden `input[name="cf-turnstile-response"]` into
/// the DOM. The token is read on demand so it can be attached to protected
/// backend requests. The component resets the widget whenever it becomes
/// visible so stale single-use tokens are discarded.
#[component]
pub fn TurnstileWidget(id: &'static str, #[prop(into)] visible: Signal<bool>) -> impl IntoView {
    Effect::new(move |_| {
        if visible.get() {
            reset_turnstile_internal();
        }
    });

    view! {
        <div
            id=id
            class="cf-turnstile"
            data-sitekey="0x4AAAAAAEA7QaTfmDX0gWZ2"
            data-action="turnstile-spin-v2"
        ></div>
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
