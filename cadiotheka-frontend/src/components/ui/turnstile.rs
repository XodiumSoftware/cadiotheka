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
pub fn TurnstileWidget(#[prop(into)] visible: Signal<bool>) -> impl IntoView {
    Effect::new(move |_| {
        if visible.get() {
            reset_turnstile_internal();
        }
    });

    view! {
        <div
            class="cf-turnstile"
            data-sitekey="0x4AAAAAAEA7QaTfmDX0gWZ2"
            data-action="turnstile-spin-v2"
            data-callback="turnstileOnSuccess"
            data-error-callback="turnstileOnError"
        ></div>
    }
}

/// Reads the current Turnstile response token from the DOM, if any.
///
/// Returns `None` when the widget has not rendered a token yet.
pub fn turnstile_response() -> Option<String> {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| {
            document
                .query_selector("input[name='cf-turnstile-response']")
                .ok()
        })
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
    let Some(window) = web_sys::window() else {
        return;
    };

    let value = js_sys::Reflect::get(&window, &JsValue::from_str("turnstile"))
        .ok()
        .filter(|value| !value.is_undefined() && !value.is_null());
    let Some(value) = value else {
        return;
    };

    let Ok(object) = value.dyn_into::<js_sys::Object>() else {
        return;
    };
    object.unchecked_ref::<TurnstileApi>().reset_opt();
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    type TurnstileApi;

    #[wasm_bindgen(method, js_name = "reset")]
    fn reset_opt(this: &TurnstileApi);
}
