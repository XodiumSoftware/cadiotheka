use leptos::prelude::*;

/// Floating isometric wireframe shapes for CAD aesthetic background
#[component]
pub fn WireframeShapes() -> impl IntoView {
    view! {
        <div class="absolute inset-0 h-full w-full pointer-events-none overflow-hidden">
            {/* Floating Cube */}
            <svg
                class="absolute wireframe-float-slow"
                style="top: 15%; left: 10%; width: 80px; height: 80px;"
                viewBox="0 0 100 100"
                xmlns="http://www.w3.org/2000/svg"
            >
                {/* Isometric cube wireframe */}
                <g stroke="var(--color-primary)" stroke-width="1" fill="none" stroke-opacity="0.4">
                    {/* Front face */}
                    <path d="M30,35 L70,35 L70,75 L30,75 Z" />
                    {/* Back face */}
                    <path d="M45,20 L85,20 L85,60 L45,60 Z" />
                    {/* Connecting edges */}
                    <path d="M30,35 L45,20" />
                    <path d="M70,35 L85,20" />
                    <path d="M70,75 L85,60" />
                    <path d="M30,75 L45,60" />
                </g>
            </svg>

            {/* Floating Cylinder */}
            <svg
                class="absolute wireframe-float-medium"
                style="top: 60%; right: 15%; width: 60px; height: 100px;"
                viewBox="0 0 60 100"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g stroke="var(--color-secondary)" stroke-width="1" fill="none" stroke-opacity="0.35">
                    {/* Top ellipse */}
                    <ellipse cx="30" cy="15" rx="20" ry="8" />
                    {/* Bottom ellipse */}
                    <ellipse cx="30" cy="85" rx="20" ry="8" />
                    {/* Side lines */}
                    <line x1="10" y1="15" x2="10" y2="85" />
                    <line x1="50" y1="15" x2="50" y2="85" />
                    {/* Hidden back curve suggestion */}
                    <path d="M10,15 A20,8 0 0,1 50,15" stroke-opacity="0.2" />
                </g>
            </svg>

            {/* Floating Cone */}
            <svg
                class="absolute wireframe-float-fast"
                style="top: 25%; right: 25%; width: 70px; height: 90px;"
                viewBox="0 0 70 90"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g stroke="var(--color-primary)" stroke-width="1" fill="none" stroke-opacity="0.3">
                    {/* Base ellipse */}
                    <ellipse cx="35" cy="75" rx="25" ry="10" />
                    {/* Apex to base edges */}
                    <line x1="35" y1="10" x2="10" y2="75" />
                    <line x1="35" y1="10" x2="60" y2="75" />
                    {/* Cross-section for depth */}
                    <line x1="35" y1="10" x2="35" y2="75" stroke-opacity="0.2" />
                </g>
            </svg>

            {/* Small floating cube */}
            <svg
                class="absolute wireframe-float-medium"
                style="bottom: 30%; left: 20%; width: 50px; height: 50px;"
                viewBox="0 0 100 100"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g stroke="var(--color-secondary)" stroke-width="1" fill="none" stroke-opacity="0.3">
                    <path d="M25,40 L65,40 L65,80 L25,80 Z" />
                    <path d="M40,25 L80,25 L80,65 L40,65 Z" />
                    <path d="M25,40 L40,25" />
                    <path d="M65,40 L80,25" />
                    <path d="M65,80 L80,65" />
                    <path d="M25,80 L40,65" />
                </g>
            </svg>

            {/* Pyramid shape */}
            <svg
                class="absolute wireframe-float-slow"
                style="top: 70%; left: 60%; width: 60px; height: 70px;"
                viewBox="0 0 60 70"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g stroke="var(--color-primary)" stroke-width="1" fill="none" stroke-opacity="0.25">
                    {/* Base square */}
                    <path d="M10,50 L30,60 L50,50 L30,40 Z" />
                    {/* Apex edges */}
                    <line x1="10" y1="50" x2="30" y2="10" />
                    <line x1="30" y1="60" x2="30" y2="10" />
                    <line x1="50" y1="50" x2="30" y2="10" />
                    <line x1="30" y1="40" x2="30" y2="10" />
                </g>
            </svg>

            {/* Corner accent - small technical markers */}
            <div class="absolute bottom-8 right-8 flex gap-4">
                <svg width="30" height="30" viewBox="0 0 30 30" class="wireframe-pulse">
                    <g stroke="var(--color-primary)" stroke-width="1" fill="none" stroke-opacity="0.5">
                        <circle cx="15" cy="15" r="10" />
                        <line x1="15" y1="5" x2="15" y2="25" />
                        <line x1="5" y1="15" x2="25" y2="15" />
                    </g>
                </svg>
            </div>

            {/* Top left accent */}
            <div class="absolute top-20 left-8">
                <svg width="24" height="24" viewBox="0 0 24 24" class="wireframe-rotate-slow">
                    <g stroke="var(--color-secondary)" stroke-width="1" fill="none" stroke-opacity="0.4">
                        <rect x="4" y="4" width="16" height="16" />
                        <rect x="8" y="8" width="8" height="8" />
                    </g>
                </svg>
            </div>
        </div>
    }
}
