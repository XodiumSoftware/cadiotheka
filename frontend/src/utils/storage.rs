//! Browser `localStorage` helpers.

use leptos::wasm_bindgen::JsValue;

/// Prefix applied to all stored keys to avoid collisions with other apps on
/// the same origin.
const PREFIX: &str = "cadiotheka.";

fn prefixed_key(key: &str) -> String {
    format!("{PREFIX}{key}")
}

/// Read a string value from `localStorage`, returning `None` if the key is
/// missing or if storage is unavailable.
pub fn local_storage_get(key: &str) -> Option<String> {
    let storage = leptos::web_sys::window()?.local_storage().ok().flatten()?;
    storage.get_item(&prefixed_key(key)).ok().flatten()
}

/// Write a string value to `localStorage`. Errors are logged to the console and
/// ignored.
pub fn local_storage_set(key: &str, value: &str) {
    let Some(storage) = leptos::web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    else {
        return;
    };
    if let Err(err) = storage.set_item(&prefixed_key(key), value) {
        leptos::web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Failed to write localStorage key '{key}': {err:?}"
        )));
    }
}

/// Remove a value from `localStorage`. Errors are logged to the console and
/// ignored.
pub fn local_storage_remove(key: &str) {
    let Some(storage) = leptos::web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    else {
        return;
    };
    if let Err(err) = storage.remove_item(&prefixed_key(key)) {
        leptos::web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Failed to remove localStorage key '{key}': {err:?}"
        )));
    }
}
