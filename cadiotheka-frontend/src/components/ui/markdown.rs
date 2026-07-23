use ammonia::{Builder, UrlRelative};
use leptos::prelude::*;
use pulldown_cmark::{Options, Parser, html};

/// Renders CommonMark markdown as a Leptos view.
///
/// Markdown is converted to HTML by `pulldown-cmark`, sanitized with `ammonia`,
/// and then styled with project-specific Tailwind classes.
#[component]
pub fn MarkdownView(#[prop(into)] source: String) -> impl IntoView {
    let source = source.clone();
    let html = leptos::prelude::Memo::new(move |_| render_markdown(&source));

    view! {
        <div class="prose prose-sm max-w-none text-base-content/80" inner_html=move || html.get()></div>
    }
}

fn render_markdown(source: &str) -> String {
    let parser = Parser::new_ext(source, Options::empty());
    let mut raw = String::new();
    html::push_html(&mut raw, parser);

    let safe = Builder::default()
        .link_rel(Some("noopener noreferrer"))
        .url_relative(UrlRelative::PassThrough)
        .clean(&raw)
        .to_string();

    apply_classes(&safe)
}

fn apply_classes(html: &str) -> String {
    html.replace("<p>", r#"<p class="mb-3">"#)
        .replace(
            "<h1>",
            r#"<h1 class="text-xl font-bold text-primary mt-4 mb-2">"#,
        )
        .replace(
            "<h2>",
            r#"<h2 class="text-lg font-bold text-primary mt-3 mb-2">"#,
        )
        .replace(
            "<h3>",
            r#"<h3 class="text-base font-semibold text-base-content mt-2 mb-1">"#,
        )
        .replace(
            "<h4>",
            r#"<h4 class="text-base font-semibold text-base-content mt-2 mb-1">"#,
        )
        .replace(
            "<h5>",
            r#"<h5 class="text-base font-semibold text-base-content mt-2 mb-1">"#,
        )
        .replace(
            "<h6>",
            r#"<h6 class="text-base font-semibold text-base-content mt-2 mb-1">"#,
        )
        .replace("<ul>", r#"<ul class="list-disc list-inside mb-3 pl-1">"#)
        .replace("<li>", r#"<li class="mb-1">"#)
        .replace("<hr>", r#"<hr class="border-base-content/10 my-3">"#)
        .replace(
            "<a ",
            r#"<a target="_blank" class="text-primary hover:underline" "#,
        )
        .replace(
            "<a>",
            r#"<a target="_blank" class="text-primary hover:underline">"#,
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_markdown_basic_formatting() {
        let html = render_markdown("**bold** and *italic*");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn render_markdown_strips_raw_html() {
        let html = render_markdown("<script>alert('x')</script>");
        assert!(!html.contains("<script>"));
        assert!(!html.contains("alert"));
    }

    #[test]
    fn render_markdown_headings_with_classes() {
        let html = render_markdown("# Heading 1\n## Heading 2\n### Heading 3");
        assert!(html.contains("<h1 class=\"text-xl font-bold text-primary mt-4 mb-2\">"));
        assert!(html.contains("</h1>"));
        assert!(html.contains("<h2 class=\"text-lg font-bold text-primary mt-3 mb-2\">"));
        assert!(
            html.contains("<h3 class=\"text-base font-semibold text-base-content mt-2 mb-1\">")
        );
    }

    #[test]
    fn render_markdown_lists_and_items() {
        let html = render_markdown("- first\n- second");
        assert!(html.contains("<ul class=\"list-disc list-inside mb-3 pl-1\">"));
        assert!(html.contains("<li class=\"mb-1\">"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn render_markdown_code_span() {
        let html = render_markdown("use `code` here");
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn render_markdown_horizontal_rule() {
        let html = render_markdown("---");
        assert!(html.contains("<hr class=\"border-base-content/10 my-3\">"));
    }

    #[test]
    fn render_markdown_links_open_in_new_tab() {
        let html = render_markdown("[example](https://example.com)");
        assert!(html.contains("<a target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains("href=\"https://example.com\""));
    }
}
