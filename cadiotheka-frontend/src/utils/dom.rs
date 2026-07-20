use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;

/// Add a listener to the browser `window` and automatically remove it when
/// the surrounding effect is cleaned up.
///
/// Returns `None` if the listener could not be registered.
pub fn window_event_listener<E, F>(event: &'static str, mut handler: F) -> Option<()>
where
    E: JsCast + 'static,
    F: FnMut(E) + 'static,
{
    let window = leptos::web_sys::window()?;
    let closure = Closure::wrap(Box::new(move |ev: leptos::web_sys::Event| {
        if let Ok(typed) = ev.dyn_into::<E>() {
            handler(typed);
        }
    }) as Box<dyn FnMut(_)>);

    // Transfer the closure to JavaScript ownership. The listener is removed in
    // `on_cleanup`; once detached, the JS function becomes unreachable and is
    // collected, freeing the associated Rust closure.
    let function: js_sys::Function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    if let Err(err) = window.add_event_listener_with_callback(event, &function) {
        leptos::web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Failed to add window '{event}' event listener: {err:?}"
        )));
        return None;
    }
    std::mem::forget(closure);

    leptos::prelude::on_cleanup(move || {
        if let Some(window) = leptos::web_sys::window()
            && let Err(err) = window.remove_event_listener_with_callback(event, &function)
        {
            leptos::web_sys::console::warn_1(&JsValue::from_str(&format!(
                "Failed to remove window '{event}' event listener: {err:?}"
            )));
        }
    });

    Some(())
}
