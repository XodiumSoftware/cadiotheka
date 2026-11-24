use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;

/// Moves focus to an element, logging instead of failing.
///
/// `focus()` throws only when the element is detached or not focusable, neither
/// of which should interrupt the interaction that requested it. The failure is
/// logged so a focus that silently stops working stays diagnosable.
///
/// Returns whether focus was applied.
pub fn focus(element: &web_sys::HtmlElement) -> bool {
    if let Err(err) = element.focus() {
        leptos::logging::warn!("failed to focus element: {err:?}");
        return false;
    }
    true
}

/// Navigates the current browsing context to `url`, logging instead of failing.
///
/// A blocked navigation is either a browser setting or an invalid URL, both of
/// which leave the page where it was; surfacing the reason beats a silent
/// no-op that looks like a broken button.
pub fn navigate_to(url: &str) {
    let Some(window) = web_sys::window() else {
        leptos::logging::warn!("no window to navigate to {url}");
        return;
    };
    if let Err(err) = window.location().set_href(url) {
        leptos::logging::warn!("failed to navigate to {url}: {err:?}");
    }
}

/// Removes a window event listener, logging instead of failing.
///
/// Listener removal only fails for a callback that was never registered, which
/// would be a bug rather than a runtime condition, so it is logged rather than
/// propagated out of a cleanup closure where nothing could act on it.
pub fn remove_window_listener(window: &web_sys::Window, event: &str, function: &js_sys::Function) {
    if let Err(err) = window.remove_event_listener_with_callback(event, function) {
        leptos::logging::warn!("failed to remove window '{event}' listener: {err:?}");
    }
}

/// Awaits a JavaScript promise, returning whether it resolved.
///
/// Several `web-sys` APIs hand back a promise instead of a typed result, so the
/// value cannot be inspected on the Rust side. Awaiting it still matters: the
/// work must finish before the caller observes its effect. Rejections are
/// logged so a promise that never resolves stays visible.
pub async fn await_promise(promise: js_sys::Promise) -> bool {
    match wasm_bindgen_futures::JsFuture::from(promise).await {
        Ok(_) => true,
        Err(err) => {
            leptos::logging::warn!("promise rejected: {err:?}");
            false
        }
    }
}

/// Add a listener to the browser `document` and automatically remove it when
/// the surrounding effect is cleaned up.
///
/// Returns `None` if the listener could not be registered.
pub fn document_event_listener<E, F>(event: &'static str, mut handler: F) -> Option<()>
where
    E: JsCast + 'static,
    F: FnMut(E) + 'static,
{
    let document = web_sys::window()?.document()?;
    let closure = Closure::wrap(Box::new(move |ev: web_sys::Event| {
        if let Ok(typed) = ev.dyn_into::<E>() {
            handler(typed);
        }
    }));

    let function: js_sys::Function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    if let Err(err) = document.add_event_listener_with_callback(event, &function) {
        web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Failed to add document '{event}' event listener: {err:?}"
        )));
        return None;
    }
    std::mem::forget(closure);

    leptos::prelude::on_cleanup(move || {
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Err(err) = document.remove_event_listener_with_callback(event, &function)
        {
            web_sys::console::warn_1(&JsValue::from_str(&format!(
                "Failed to remove document '{event}' event listener: {err:?}"
            )));
        }
    });

    Some(())
}

/// Add a listener to the browser `window` and automatically remove it when
/// the surrounding effect is cleaned up.
///
/// Returns `None` if the listener could not be registered.
pub fn window_event_listener<E, F>(event: &'static str, mut handler: F) -> Option<()>
where
    E: JsCast + 'static,
    F: FnMut(E) + 'static,
{
    let window = web_sys::window()?;
    let closure = Closure::wrap(Box::new(move |ev: web_sys::Event| {
        if let Ok(typed) = ev.dyn_into::<E>() {
            handler(typed);
        }
    }));

    let function: js_sys::Function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    if let Err(err) = window.add_event_listener_with_callback(event, &function) {
        web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Failed to add window '{event}' event listener: {err:?}"
        )));
        return None;
    }
    std::mem::forget(closure);

    leptos::prelude::on_cleanup(move || {
        if let Some(window) = web_sys::window()
            && let Err(err) = window.remove_event_listener_with_callback(event, &function)
        {
            web_sys::console::warn_1(&JsValue::from_str(&format!(
                "Failed to remove window '{event}' event listener: {err:?}"
            )));
        }
    });

    Some(())
}
