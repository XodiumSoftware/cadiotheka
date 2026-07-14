use crate::components::animations::linedraw::LineDrawHero;
use crate::components::effects::blueprintgrid::BlueprintGrid;
use crate::components::effects::parallax::ParallaxLanding;
use crate::components::effects::wireframes::WireframeShapes;
use crate::components::ui::codeblock::CodeBlock;
use leptos::prelude::*;

#[component]
pub fn LandingSection() -> impl IntoView {
    view! {
        <section id="landing" class="relative isolate px-6 py-[8dvh] sm:py-[10dvh] lg:px-8">
            <BlueprintGrid />
            <WireframeShapes />
            <LineDrawHero />
            <ParallaxLanding />

            <div
                class="absolute inset-x-0 -top-40 -z-10 transform-gpu overflow-hidden blur-3xl sm:-top-80"
                aria-hidden="true"
            >
                <div class="relative left-[calc(50%-11rem)] aspect-[1155/678] w-[36.125rem] -translate-x-1/2 rotate-[30deg] bg-gradient-to-tr from-primary to-secondary opacity-30 sm:left-[calc(50%-30rem)] sm:w-[72.1875rem] clip-organic-blob"></div>
            </div>

            <div class="mx-auto max-w-2xl py-[6dvh] sm:py-[8dvh] lg:py-[10dvh]">
                <CodeBlock />
            </div>

            <div
                class="absolute inset-x-0 top-[calc(100%-13rem)] -z-10 transform-gpu overflow-hidden blur-3xl sm:top-[calc(100%-50rem)]"
                aria-hidden="true"
            >
                <div class="relative left-[calc(50%+3rem)] aspect-[1155/678] w-[36.125rem] -translate-x-1/2 bg-gradient-to-tr from-primary to-secondary opacity-30 sm:left-[calc(50%+36rem)] sm:w-[72.1875rem] clip-organic-blob"></div>
            </div>
        </section>
    }
}
