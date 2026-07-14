use leptos::prelude::*;

/// Hexagon background pattern for sections
#[component]
pub fn HexPattern(#[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <div class={format!("absolute inset-0 pointer-events-none overflow-hidden {}", class)}>
            <svg
                class="absolute inset-0 w-full h-full opacity-5"
                xmlns="http://www.w3.org/2000/svg"
            >
                <defs>
                    <pattern
                        id="hexPattern"
                        width="56"
                        height="100"
                        patternUnits="userSpaceOnUse"
                    >
                        <path
                            d="M28 66L0 50L0 16L28 0L56 16L56 50L28 66L28 100"
                            fill="none"
                            stroke="var(--color-primary)"
                            stroke-width="1"
                        />
                        <path
                            d="M28 0L28 34L0 50L0 84L28 100L56 84L56 50L28 34"
                            fill="none"
                            stroke="var(--color-primary)"
                            stroke-width="1"
                        />
                    </pattern>
                </defs>
                <rect width="100%" height="100%" fill="url(#hexPattern)" />
            </svg>
        </div>
    }
}
