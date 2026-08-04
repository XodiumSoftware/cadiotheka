use ammonia::{Builder, UrlRelative};
use leptos::prelude::*;
use pulldown_cmark::{Options, Parser, html};

/// Renders CommonMark markdown as a Leptos view.
///
/// Markdown is converted to HTML by `pulldown-cmark`, then sanitized and styled
/// with project-specific Tailwind classes by `ammonia`, which injects the class
/// attributes during parsing rather than string-matching the rendered markup.
#[component]
pub fn MarkdownView(#[prop(into)] source: String) -> impl IntoView {
    let html = leptos::prelude::Memo::new(move |_| render_markdown(&source));

    view! {
        <div class="prose prose-sm max-w-none text-base-content/80" inner_html=move || html.get()></div>
    }
}

fn render_markdown(source: &str) -> String {
    let parser = Parser::new_ext(source, Options::empty());
    let mut raw = String::new();
    html::push_html(&mut raw, parser);

    style_markdown(&raw)
}

/// Sanitizes raw HTML with `ammonia` while forcing project Tailwind classes and
/// link behavior onto the styled tags.
///
/// `class` is whitelisted per tag and its value forced via
/// [`Builder::set_tag_attribute_value`], so any user-supplied `class` attribute
/// in the source is overridden rather than trusted.
fn style_markdown(html: &str) -> String {
    Builder::default()
        .link_rel(Some("noopener noreferrer"))
        .url_relative(UrlRelative::PassThrough)
        .add_tag_attributes("p", &["class"])
        .set_tag_attribute_value("p", "class", "mb-3")
        .add_tag_attributes("h1", &["class"])
        .set_tag_attribute_value("h1", "class", "text-xl font-bold text-primary mt-4 mb-2")
        .add_tag_attributes("h2", &["class"])
        .set_tag_attribute_value("h2", "class", "text-lg font-bold text-primary mt-3 mb-2")
        .add_tag_attributes("h3", &["class"])
        .set_tag_attribute_value(
            "h3",
            "class",
            "text-base font-semibold text-base-content mt-2 mb-1",
        )
        .add_tag_attributes("h4", &["class"])
        .set_tag_attribute_value(
            "h4",
            "class",
            "text-base font-semibold text-base-content mt-2 mb-1",
        )
        .add_tag_attributes("h5", &["class"])
        .set_tag_attribute_value(
            "h5",
            "class",
            "text-base font-semibold text-base-content mt-2 mb-1",
        )
        .add_tag_attributes("h6", &["class"])
        .set_tag_attribute_value(
            "h6",
            "class",
            "text-base font-semibold text-base-content mt-2 mb-1",
        )
        .add_tag_attributes("ul", &["class"])
        .set_tag_attribute_value("ul", "class", "list-disc list-inside mb-3 pl-1")
        .add_tag_attributes("li", &["class"])
        .set_tag_attribute_value("li", "class", "mb-1")
        .add_tag_attributes("hr", &["class"])
        .set_tag_attribute_value("hr", "class", "border-base-content/10 my-3")
        .add_tag_attributes("a", &["target", "class"])
        .set_tag_attribute_value("a", "target", "_blank")
        .set_tag_attribute_value("a", "class", "text-primary hover:underline")
        .clean(html)
        .to_string()
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
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("class=\"text-primary hover:underline\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains("href=\"https://example.com\""));
    }

    #[test]
    fn render_markdown_overrides_user_class_attribute() {
        let html = render_markdown("<p class=\"injected\">text</p>");
        assert!(!html.contains("injected"));
        assert!(html.contains("<p class=\"mb-3\">"));
    }
}
