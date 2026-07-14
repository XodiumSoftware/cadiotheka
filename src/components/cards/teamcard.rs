use crate::components::ui::cornerframe::CornerFrame;
use leptos::prelude::*;

#[derive(Clone)]
pub struct TeamCardProperties {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
    pub role: Option<String>,
}

#[component]
pub fn TeamCard(props: TeamCardProperties) -> impl IntoView {
    view! {
        <a
            href=props.html_url
            target="_blank"
            rel="noopener noreferrer"
            class="group flex flex-col items-center justify-center h-full w-full bg-ghost hover:border-primary hover:text-primary border-2 border-base-content/80 p-2"
        >
            <CornerFrame style="square" class="h-full w-full flex flex-col items-center justify-center">
                <div class="avatar mb-4">
                    <div class="w-20 rounded-full ring-2 ring-primary ring-offset-2 ring-offset-base-100">
                        <img
                            src=props.avatar_url
                            alt=props.login.clone()
                            width="80"
                            height="80"
                            loading="lazy"
                            decoding="async"
                        />
                    </div>
                </div>
                <span class="font-bold text-lg">{props.login}</span>
                {props.role.map(|role| view! {
                    <span class="text-sm text-base-content/60 mt-1">
                        {role}
                    </span>
                })}
            </CornerFrame>
        </a>
    }
}
