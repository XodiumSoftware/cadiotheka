use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;

/// Wraps a `wasm_bindgen::closure::Closure` so it can be used with APIs
/// that require `Send + Sync` (like Leptos `on_cleanup`).
///
/// # Safety
///
/// This is sound because WASM is single-threaded; there is only ever one
/// thread of execution, so `Send` and `Sync` are trivially satisfied.
pub struct SendWrapper<T>(pub T);

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T: JsCast> SendWrapper<T> {
    /// Convert the wrapped closure to a `js_sys::Function` reference.
    ///
    /// # Panics
    ///
    /// Panics if the inner value cannot be cast to `Function`.
    pub fn as_function(&self) -> js_sys::Function {
        self.0.as_ref().unchecked_ref::<js_sys::Function>().clone()
    }
}

/// Check whether the user has requested reduced motion.
///
/// Defaults to `false` if the media query cannot be evaluated.
pub fn prefers_reduced_motion() -> bool {
    leptos::web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
        .flatten()
        .is_some_and(|mql| mql.matches())
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
    let window = leptos::web_sys::window()?;
    let closure = SendWrapper(Closure::wrap(Box::new(move |ev: leptos::web_sys::Event| {
        if let Ok(typed) = ev.dyn_into::<E>() {
            handler(typed);
        }
    }) as Box<dyn FnMut(_)>));

    let fn_ref: js_sys::Function = closure
        .0
        .as_ref()
        .unchecked_ref::<js_sys::Function>()
        .clone();
    window
        .add_event_listener_with_callback(event, &fn_ref)
        .ok()?;

    leptos::prelude::on_cleanup(move || {
        if let Some(window) = leptos::web_sys::window() {
            let _ = window.remove_event_listener_with_callback(event, &fn_ref);
        }
        drop(closure);
    });

    Some(())
}

/// Observe the intersection of a set of elements and call the provided
/// callback whenever the observed state changes.
///
/// `threshold` is a value between 0.0 and 1.0 that controls how much of an
/// element must be visible before the observer fires. A threshold of 0.0
/// fires as soon as a single pixel is visible; 1.0 requires the entire
/// element to be visible.
///
/// The observer is disconnected when the surrounding effect is cleaned up.
/// Returns `None` if the observer could not be created.
pub fn observe_intersections<F>(
    elements: &[leptos::web_sys::Element],
    threshold: f64,
    mut callback: F,
) -> Option<()>
where
    F: FnMut(&[leptos::web_sys::IntersectionObserverEntry]) + 'static,
{
    let window = leptos::web_sys::window()?;

    let closure = SendWrapper(Closure::wrap(Box::new(move |entries: js_sys::Array| {
        let typed: Vec<leptos::web_sys::IntersectionObserverEntry> = entries
            .iter()
            .filter_map(|entry| {
                entry
                    .dyn_into::<leptos::web_sys::IntersectionObserverEntry>()
                    .ok()
            })
            .collect();
        callback(&typed);
    }) as Box<dyn FnMut(_)>));
    let _ = &window; // keep the window reference alive for the closure lifetime

    let options = leptos::web_sys::IntersectionObserverInit::new();
    let threshold = threshold.clamp(0.0, 1.0);
    options.set_threshold(&js_sys::Array::of1(&js_sys::Number::from(threshold)));

    let observer = leptos::web_sys::IntersectionObserver::new_with_options(
        closure.0.as_ref().unchecked_ref(),
        &options,
    )
    .ok()?;

    for element in elements {
        observer.observe(element);
    }

    leptos::prelude::on_cleanup(move || {
        observer.disconnect();
        drop(closure);
    });

    Some(())
}

/// Strip a `-dirty` suffix from a Git SHA, if present.
pub fn clean_sha(sha: &str) -> &str {
    sha.strip_suffix("-dirty").unwrap_or(sha)
}

/// Return a Tailwind color class for a programming language name.
///
/// Unknown languages fall back to a neutral base-content badge.
pub fn language_color(language: &str) -> &'static str {
    match language {
        "Rust" => "bg-[#dea584]",
        "TypeScript" => "bg-[#3178c6]",
        "JavaScript" => "bg-[#f1e05a]",
        "Python" => "bg-[#3572A5]",
        "HTML" => "bg-[#e34c26]",
        "CSS" => "bg-[#563d7c]",
        "Java" | "java" => "bg-[#b07219]",
        "Go" => "bg-[#00ADD8]",
        "C" => "bg-[#555555]",
        "C++" => "bg-[#f34b7d]",
        "Kotlin" => "bg-[#A97BFF]",
        _ => "bg-base-content/50",
    }
}
