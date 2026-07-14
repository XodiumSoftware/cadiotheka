use leptos::prelude::*;

/// CAD line drawing animation - SVG path that draws like a CAD line being drafted
/// Uses CSS animation triggered on component mount
#[component]
pub fn LineDraw(#[prop(default = "")] class: &'static str) -> impl IntoView {
    view! {
        <div class={format!("relative w-full h-16 overflow-hidden {}", class)}>
            <svg
                class="absolute inset-0 w-full h-full"
                viewBox="0 0 1200 60"
                preserveAspectRatio="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                // Static underlay line (dim)
                <line
                    x1="0" y1="30" x2="1200" y2="30"
                    stroke="var(--color-primary)"
                    stroke-width="1"
                    stroke-opacity="0.1"
                />

                // Animated drawing line
                <line
                    x1="0" y1="30" x2="1200" y2="30"
                    class="cad-line-draw"
                    stroke="var(--color-primary)"
                    stroke-width="2"
                    stroke-linecap="round"
                />

                // Dimension markers at ends
                <g stroke="var(--color-primary)" stroke-width="1" fill="none" stroke-opacity="0.5">
                    // Left marker
                    <line x1="20" y1="25" x2="20" y2="35" />
                    <line x1="15" y1="30" x2="25" y2="30" />

                    // Right marker
                    <line x1="1180" y1="25" x2="1180" y2="35" />
                    <line x1="1175" y1="30" x2="1185" y2="30" />
                </g>

                // Measurement markers
                <g stroke="var(--color-primary)" stroke-width="1" fill="none" stroke-opacity="0.3">
                    <line x1="300" y1="25" x2="300" y2="35" />
                    <line x1="600" y1="25" x2="600" y2="35" />
                    <line x1="900" y1="25" x2="900" y2="35" />
                </g>
            </svg>
        </div>
    }
}

/// Animated line drawing for hero section on page load
#[component]
pub fn LineDrawHero() -> impl IntoView {
    view! {
        <div class="absolute inset-0 pointer-events-none overflow-hidden">
            // Decorative CAD line drawing across hero
            <svg
                class="absolute top-1/2 left-0 w-full h-32 -translate-y-1/2"
                viewBox="0 0 1200 120"
                preserveAspectRatio="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                // Main horizontal construction line
                <line
                    x1="0" y1="60" x2="1200" y2="60"
                    class="cad-line-draw-hero"
                    stroke="var(--color-primary)"
                    stroke-width="1.5"
                    stroke-opacity="0.3"
                />

                // Vertical marker lines
                <g stroke="var(--color-primary)" stroke-width="1" stroke-opacity="0.2">
                    <line x1="100" y1="55" x2="100" y2="65" />
                    <line x1="300" y1="55" x2="300" y2="65" />
                    <line x1="600" y1="55" x2="600" y2="65" />
                    <line x1="900" y1="55" x2="900" y2="65" />
                    <line x1="1100" y1="55" x2="1100" y2="65" />
                </g>

                // Arrow heads at ends
                <g fill="none" stroke="var(--color-primary)" stroke-width="1" stroke-opacity="0.4">
                    <path d="M 50,60 L 60,55 M 50,60 L 60,65" />
                    <path d="M 1150,60 L 1140,55 M 1150,60 L 1140,65" />
                </g>
            </svg>
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
    fn test_line_draw_mounts() {
        mount_to_body(|| view! { <LineDraw class="test-line" /> });
        let document = web_sys::window().unwrap().document().unwrap();
        assert!(
            document.query_selector("svg").unwrap().is_some(),
            "LineDraw should render an SVG"
        );
    }

    #[wasm_bindgen_test]
    fn test_line_draw_hero_mounts() {
        mount_to_body(LineDrawHero);
        let document = web_sys::window().unwrap().document().unwrap();
        assert!(
            document.query_selector("svg").unwrap().is_some(),
            "LineDrawHero should render an SVG"
        );
    }
}
