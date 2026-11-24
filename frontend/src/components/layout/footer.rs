use crate::components::ui::icons::{buy_me_a_coffee::BuyMeACoffeeIcon, github::GitHubIcon};
use leptos::prelude::*;

/// A single text link closing the footer.
struct FooterLink {
    /// Destination URL.
    href: &'static str,
    /// Uppercase label shown in the footer.
    label: &'static str,
    /// Whether the link leaves this site, which decides `target` and `rel`.
    external: bool,
}

/// Text links shown before the icon links, in reading order.
const FOOTER_LINKS: &[FooterLink] = &[
    FooterLink {
        href: "https://github.com/XodiumSoftware",
        label: "ABOUT",
        external: true,
    },
    FooterLink {
        href: "https://www.gnu.org/licenses/agpl-3.0.html",
        label: "LICENSING",
        external: true,
    },
    FooterLink {
        href: "mailto:info@xodium.org",
        label: "CONTACT",
        external: false,
    },
];

/// A footer entry rendered as an icon instead of a text label.
struct FooterIconLink {
    /// Destination URL opened in a new tab.
    href: &'static str,
    /// Accessible name, since the icon itself is decorative.
    aria_label: &'static str,
    /// Renders the link's icon.
    icon: fn() -> AnyView,
}

/// Icon links closing the footer, after [`FOOTER_LINKS`].
const FOOTER_ICON_LINKS: &[FooterIconLink] = &[
    FooterIconLink {
        href: "https://github.com/XodiumSoftware/cadiotheka",
        aria_label: "Cadiotheka on GitHub",
        icon: || view! { <GitHubIcon class="h-[0.9em] w-auto" /> }.into_any(),
    },
    FooterIconLink {
        href: "https://buymeacoffee.com/illyrius",
        aria_label: "Buy me a coffee",
        icon: || view! { <BuyMeACoffeeIcon class="h-[0.9em] w-auto" /> }.into_any(),
    },
];

/// Site footer: one line of small monospace notes closing the page, in the
/// same style as xodium.org.
///
/// The wordmark is plain text, not a link: it sits on the page it would point
/// to. The icon links carry their own accessible names, since the marks are
/// decorative and sized in `em` so they track the row's font size. The row
/// wraps rather than scrolls on narrow screens.
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="px-6 lg:px-8 py-6" aria-label="Site footer">
            <div class="mx-auto max-w-7xl flex flex-wrap items-baseline justify-center gap-x-2 gap-y-1 font-mono text-[0.65rem] uppercase tracking-[0.08em] text-base-content opacity-75 max-sm:text-[0.55rem]">
                <span>"Cadiotheka™"</span>
                {FOOTER_LINKS.iter().map(|link| view! {
                    <>
                        <span class="opacity-50 select-none" aria-hidden="true">"·"</span>
                        <a
                            href=link.href
                            target=link.external.then_some("_blank")
                            rel=link.external.then_some("noopener noreferrer")
                            class="font-[inherit] tracking-[inherit] text-current uppercase underline underline-offset-[3px] transition-colors hover:text-primary"
                        >
                            {link.label}
                        </a>
                    </>
                }).collect::<Vec<_>>()}
                {FOOTER_ICON_LINKS.iter().map(|link| view! {
                    <>
                        <span class="opacity-50 select-none" aria-hidden="true">"·"</span>
                        <a
                            href=link.href
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center self-center text-current no-underline transition-colors hover:text-primary"
                            aria-label=link.aria_label
                        >
                            {(link.icon)()}
                        </a>
                    </>
                }).collect::<Vec<_>>()}
            </div>
        </footer>
    }
}
