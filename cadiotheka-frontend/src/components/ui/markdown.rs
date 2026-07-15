use leptos::prelude::*;
use pulldown_cmark::{Event, Parser, Tag as CmarkTag, TagEnd};

/// Renders a subset of CommonMark markdown as a Leptos view.
///
/// Supports paragraphs, headings, bold, italic, links, lists, and code spans.
/// Raw HTML and unknown tags are escaped and rendered as plain text.
#[component]
pub fn MarkdownView(#[prop(into)] source: String) -> impl IntoView {
    let source = source.clone();
    let html = leptos::prelude::Memo::new(move |_| render_markdown(&source));

    view! {
        <div class="prose prose-sm max-w-none text-base-content/80" inner_html=move || html.get()></div>
    }
}

fn render_markdown(source: &str) -> String {
    let parser = Parser::new(source);
    let mut output = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                CmarkTag::Paragraph => output.push_str("<p class=\"mb-3\">"),
                CmarkTag::Heading { level, .. } => {
                    let class = match level {
                        pulldown_cmark::HeadingLevel::H1 => {
                            "text-xl font-bold text-primary mt-4 mb-2"
                        }
                        pulldown_cmark::HeadingLevel::H2 => {
                            "text-lg font-bold text-primary mt-3 mb-2"
                        }
                        _ => "text-base font-semibold text-base-content mt-2 mb-1",
                    };
                    output.push_str(&format!("<h{} class=\"{}\">", level as u8, class));
                }
                CmarkTag::List(_) => {
                    output.push_str("<ul class=\"list-disc list-inside mb-3 pl-1\">")
                }
                CmarkTag::Item => output.push_str("<li class=\"mb-1\">"),
                CmarkTag::Emphasis => output.push_str("<em>"),
                CmarkTag::Strong => output.push_str("<strong>"),
                CmarkTag::Link { dest_url, .. } => {
                    output.push_str(&format!(
                        "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"text-primary hover:underline\">",
                        html_escape(&dest_url)
                    ));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => output.push_str("</p>"),
                TagEnd::Heading(level) => output.push_str(&format!("</h{}\u{003e}", level as u8)),
                TagEnd::List(_) => output.push_str("</ul>"),
                TagEnd::Item => output.push_str("</li>"),
                TagEnd::Emphasis => output.push_str("</em>"),
                TagEnd::Strong => output.push_str("</strong>"),
                TagEnd::Link => {
                    output.push_str("</a>");
                }
                _ => {}
            },
            Event::Text(text) => output.push_str(&html_escape(&text)),
            Event::Code(code) => output.push_str(&html_escape(&code)),
            Event::Html(html) | Event::InlineHtml(html) => output.push_str(&html_escape(&html)),
            Event::SoftBreak | Event::HardBreak => output.push(' '),
            Event::Rule => output.push_str("<hr class=\"border-base-content/10 my-3\">"),
            _ => {}
        }
    }

    output
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;")
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
    fn render_markdown_escapes_html() {
        let html = render_markdown("<script>alert('x')</script>");
        assert!(!html.contains("<script>"));
        assert!(html.contains("alert(&#x27;x&#x27;)"));
    }

    #[test]
    fn render_markdown_link_renders_with_target_blank() {
        let html = render_markdown("[link](https://example.com)");
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("target=\"_blank\""));
    }
}
