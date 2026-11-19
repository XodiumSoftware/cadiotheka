use leptos::prelude::*;

const GITHUB_REPO_URL: &str = "https://github.com/XodiumSoftware/cadiotheka";

/// A single text link closing the footer.
struct FooterLink {
    /// Destination URL.
    href: &'static str,
    /// Uppercase label shown in the footer.
    label: &'static str,
    /// Whether the link leaves this site, which decides `target` and `rel`.
    external: bool,
}

/// Footer links in reading order, after the wordmark.
const FOOTER_LINKS: &[FooterLink] = &[
    FooterLink {
        href: GITHUB_REPO_URL,
        label: "GitHub",
        external: true,
    },
    FooterLink {
        href: "https://www.gnu.org/licenses/agpl-3.0.html",
        label: "Licensing",
        external: true,
    },
    FooterLink {
        href: "mailto:info@xodium.org",
        label: "Contact",
        external: false,
    },
];

/// Site footer: one line of small monospace notes closing the page, in the
/// same style as xodium.org.
///
/// The row wraps rather than scrolls on narrow screens, so the links drop onto
/// their own line instead of forcing the page wider.
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="px-6 lg:px-8 py-6" aria-label="Site footer">
            <div class="mx-auto max-w-7xl flex flex-wrap items-baseline justify-center gap-x-2 gap-y-1 font-mono text-[0.65rem] uppercase tracking-[0.08em] text-base-content opacity-75 max-sm:text-[0.55rem]">
                <a href="/" class="font-[inherit] tracking-[inherit] text-current uppercase underline underline-offset-[3px] transition-colors hover:text-primary">
                    "Cadiotheka™"
                </a>
                {FOOTER_LINKS.iter().map(|link| {
                    view! {
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
                    }
                }).collect::<Vec<_>>()}
            </div>
        </footer>
    }
}
