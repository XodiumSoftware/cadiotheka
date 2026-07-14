use crate::utils::{prefers_reduced_motion, window_event_listener};
use leptos::prelude::*;
use leptos::web_sys;

/// Parallax background for the landing section.
/// Respects `prefers-reduced-motion` by disabling all layer transforms.
#[component]
pub fn ParallaxLanding() -> impl IntoView {
    let (scroll_y, set_scroll_y) = signal(0.0);
    let (reduced_motion, set_reduced_motion) = signal(false);

    // Detect prefers-reduced-motion at mount
    Effect::new(move |_| {
        set_reduced_motion.set(prefers_reduced_motion());
    });

    // Set up scroll listener only when motion is not reduced
    Effect::new(move |_| {
        if reduced_motion.get() {
            return;
        }

        if let Some(window) = web_sys::window()
            && let Ok(scroll) = window.scroll_y()
        {
            set_scroll_y.set(scroll);
        }

        window_event_listener::<web_sys::Event, _>("scroll", move |_ev| {
            if let Some(w) = web_sys::window()
                && let Ok(scroll) = w.scroll_y()
            {
                set_scroll_y.set(scroll);
            }
        });
    });

    // Different layers move at different speeds — disabled when reduced motion is preferred
    let layer1_transform = move || {
        if reduced_motion.get() {
            String::new()
        } else {
            format!("translateY({}px)", scroll_y.get().mul_add(0.1, 0.0))
        }
    };
    let layer2_transform = move || {
        if reduced_motion.get() {
            String::new()
        } else {
            format!("translateY({}px)", scroll_y.get().mul_add(0.3, 0.0))
        }
    };
    let layer3_transform = move || {
        if reduced_motion.get() {
            String::new()
        } else {
            format!("translateY({}px)", scroll_y.get().mul_add(0.5, 0.0))
        }
    };

    view! {
        <div class="absolute inset-0 pointer-events-none overflow-hidden">
            {/* Slowest layer - far background */}
            <div
                class="absolute inset-0 will-change-transform"
                style:transform=layer1_transform
            >
                <div class="absolute top-20 left-10 w-32 h-32 border border-primary/10 rotate-45" />
                <div class="absolute bottom-40 right-20 w-24 h-24 border border-secondary/10 rotate-12" />
            </div>

            {/* Medium layer */}
            <div
                class="absolute inset-0 will-change-transform"
                style:transform=layer2_transform
            >
                <div class="absolute top-1/3 left-1/4 w-2 h-2 bg-primary/20 rounded-full" />
                <div class="absolute top-2/3 right-1/3 w-3 h-3 bg-secondary/20 rounded-full" />
                <div class="absolute top-1/2 left-3/4 w-1 h-20 bg-primary/10" />
            </div>

            {/* Fast layer - closest */}
            <div
                class="absolute inset-0 will-change-transform"
                style:transform=layer3_transform
            >
                <div class="absolute top-1/4 right-10 text-primary/5 font-mono text-xs">
                    "X: 1024 | Y: 768"
                </div>
                <div class="absolute bottom-1/4 left-10 text-secondary/5 font-mono text-xs">
                    "SCALE: 1.0"
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::mount::mount_to_body;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_parallax_landing_mounts() {
        mount_to_body(ParallaxLanding);
        let document = web_sys::window().unwrap().document().unwrap();
        assert!(
            document
                .query_selector(".will-change-transform")
                .unwrap()
                .is_some(),
            "ParallaxLanding should render transform layers"
        );
    }
}
