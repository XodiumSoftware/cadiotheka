use leptos::prelude::*;

/// Blueprint-style grid background with measurement ticks and coordinate axes
/// Designed to be placed behind the hero section for CAD aesthetic
#[component]
pub fn BlueprintGrid() -> impl IntoView {
    view! {
        <div class="absolute inset-0 h-full w-full pointer-events-none overflow-hidden">
            {/* Base grid pattern */}
            <div class="absolute inset-0 blueprint-grid" />

            {/* Coordinate axes */}
            <svg
                class="absolute inset-0 w-full h-full"
                xmlns="http://www.w3.org/2000/svg"
            >
                {/* X and Y axes */}
                <line
                    x1="0" y1="50%" x2="100%" y2="50%"
                    stroke="var(--color-primary)"
                    stroke-width="1"
                    stroke-opacity="0.3"
                />
                <line
                    x1="50%" y1="0" x2="50%" y2="100%"
                    stroke="var(--color-primary)"
                    stroke-width="1"
                    stroke-opacity="0.3"
                />

                {/* X-axis tick marks and labels */}
                {vec![
                    ("10%", "10"), ("20%", "20"), ("30%", "30"), ("40%", "40"),
                    ("60%", "60"), ("70%", "70"), ("80%", "80"), ("90%", "90")
                ].into_iter().map(|(pos, label)| view! {
                    <>
                        <line
                            x1={pos} y1="48%" x2={pos} y2="52%"
                            stroke="var(--color-primary)"
                            stroke-width="1"
                            stroke-opacity="0.4"
                        />
                        <text
                            x={pos} y="55%"
                            fill="var(--color-primary)"
                            fill-opacity="0.5"
                            font-size="10"
                            font-family="monospace"
                            text-anchor="middle"
                        >{label}</text>
                    </>
                }).collect_view()}

                {/* Y-axis tick marks and labels */}
                {vec![
                    ("10%", "10"), ("20%", "20"), ("30%", "30"), ("40%", "40"),
                    ("60%", "60"), ("70%", "70"), ("80%", "80"), ("90%", "90")
                ].into_iter().map(|(pos, label)| view! {
                    <>
                        <line
                            x1="48%" y1={pos} x2="52%" y2={pos}
                            stroke="var(--color-primary)"
                            stroke-width="1"
                            stroke-opacity="0.4"
                        />
                        <text
                            x="45%" y={pos}
                            fill="var(--color-primary)"
                            fill-opacity="0.5"
                            font-size="10"
                            font-family="monospace"
                            text-anchor="end"
                            dominant-baseline="middle"
                        >{label}</text>
                    </>
                }).collect_view()}

                {/* Origin label */}
                <text
                    x="49%" y="54%"
                    fill="var(--color-primary)"
                    fill-opacity="0.6"
                    font-size="12"
                    font-family="monospace"
                    text-anchor="end"
                >"0,0"</text>

                {/* Corner markers - CAD style */}
                <path
                    d="M 20,20 L 40,20 M 20,20 L 20,40"
                    stroke="var(--color-primary)"
                    stroke-width="1.5"
                    stroke-opacity="0.3"
                    fill="none"
                />
                <path
                    d="M 1180,20 L 1160,20 M 1180,20 L 1180,40"
                    stroke="var(--color-primary)"
                    stroke-width="1.5"
                    stroke-opacity="0.3"
                    fill="none"
                />
                <path
                    d="M 20,580 L 40,580 M 20,580 L 20,560"
                    stroke="var(--color-primary)"
                    stroke-width="1.5"
                    stroke-opacity="0.3"
                    fill="none"
                />
                <path
                    d="M 1180,580 L 1160,580 M 1180,580 L 1180,560"
                    stroke="var(--color-primary)"
                    stroke-width="1.5"
                    stroke-opacity="0.3"
                    fill="none"
                />
            </svg>

            {/* Subtle radial mask to fade edges */}
            <div class="absolute inset-0 blueprint-mask" />
        </div>
    }
}
