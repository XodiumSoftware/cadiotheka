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

/// Requests fullscreen for the given HTML element, using vendor prefixes if needed.
///
/// # Errors
/// Returns an error if the fullscreen method exists but fails to execute.
pub fn request_fullscreen(element: &leptos::web_sys::Element) -> Result<(), JsValue> {
    if js_sys::Reflect::has(element, &JsValue::from_str("requestFullscreen")).unwrap_or(false) {
        js_sys::Reflect::apply(
            &js_sys::Reflect::get(element, &JsValue::from_str("requestFullscreen"))?
                .dyn_into::<js_sys::Function>()?,
            element,
            &js_sys::Array::new(),
        )?;
    } else if js_sys::Reflect::has(element, &JsValue::from_str("webkitRequestFullscreen"))
        .unwrap_or(false)
    {
        js_sys::Reflect::apply(
            &js_sys::Reflect::get(element, &JsValue::from_str("webkitRequestFullscreen"))?
                .dyn_into::<js_sys::Function>()?,
            element,
            &js_sys::Array::new(),
        )?;
    } else if js_sys::Reflect::has(element, &JsValue::from_str("mozRequestFullScreen"))
        .unwrap_or(false)
    {
        js_sys::Reflect::apply(
            &js_sys::Reflect::get(element, &JsValue::from_str("mozRequestFullScreen"))?
                .dyn_into::<js_sys::Function>()?,
            element,
            &js_sys::Array::new(),
        )?;
    }
    Ok(())
}

/// Exits fullscreen mode if it is currently active.
///
/// # Errors
/// Returns an error if the exit method exists but fails to execute.
pub fn exit_fullscreen() -> Result<(), JsValue> {
    let Some(document) = leptos::web_sys::window().and_then(|w| w.document()) else {
        return Ok(());
    };

    if js_sys::Reflect::has(&document, &JsValue::from_str("fullscreenElement")).unwrap_or(false)
        && !js_sys::Reflect::get(&document, &JsValue::from_str("fullscreenElement"))
            .unwrap_or(JsValue::NULL)
            .is_null()
    {
        js_sys::Reflect::apply(
            &js_sys::Reflect::get(&document, &JsValue::from_str("exitFullscreen"))?
                .dyn_into::<js_sys::Function>()?,
            &document,
            &js_sys::Array::new(),
        )?;
    } else if js_sys::Reflect::has(&document, &JsValue::from_str("webkitFullscreenElement"))
        .unwrap_or(false)
        && !js_sys::Reflect::get(&document, &JsValue::from_str("webkitFullscreenElement"))
            .unwrap_or(JsValue::NULL)
            .is_null()
    {
        js_sys::Reflect::apply(
            &js_sys::Reflect::get(&document, &JsValue::from_str("webkitExitFullscreen"))?
                .dyn_into::<js_sys::Function>()?,
            &document,
            &js_sys::Array::new(),
        )?;
    } else if js_sys::Reflect::has(&document, &JsValue::from_str("mozFullScreenElement"))
        .unwrap_or(false)
        && !js_sys::Reflect::get(&document, &JsValue::from_str("mozFullScreenElement"))
            .unwrap_or(JsValue::NULL)
            .is_null()
    {
        js_sys::Reflect::apply(
            &js_sys::Reflect::get(&document, &JsValue::from_str("mozCancelFullScreen"))?
                .dyn_into::<js_sys::Function>()?,
            &document,
            &js_sys::Array::new(),
        )?;
    }
    Ok(())
}

/// Returns whether any element is currently in fullscreen.
pub fn is_fullscreen() -> bool {
    let Some(document) = leptos::web_sys::window().and_then(|w| w.document()) else {
        return false;
    };

    [
        "fullscreenElement",
        "webkitFullscreenElement",
        "mozFullScreenElement",
    ]
    .iter()
    .any(|name| {
        js_sys::Reflect::get(&document, &JsValue::from_str(name))
            .ok()
            .is_some_and(|v| !v.is_null())
    })
}

/// Toggles fullscreen mode for the given element.
///
/// # Errors
/// Returns an error if entering or exiting fullscreen fails.
pub fn toggle_fullscreen(element: &leptos::web_sys::Element) -> Result<(), JsValue> {
    if is_fullscreen() {
        exit_fullscreen()
    } else {
        request_fullscreen(element)
    }
}
