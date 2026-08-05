use leptos::prelude::*;

/// Number of page buttons shown on each side of the current page.
const DEFAULT_SIBLING_COUNT: usize = 1;

/// A single page number or an ellipsis gap in a pagination bar.
enum PageItem {
    Page(usize),
    Ellipsis,
}

/// Builds the ordered list of page numbers and ellipsis markers to display.
fn pagination_pages(current: usize, total: usize, sibling_count: usize) -> Vec<PageItem> {
    if total == 0 {
        return Vec::new();
    }

    let window_size = (sibling_count * 2 + 1).min(total);
    let start = if current + sibling_count >= total {
        total.saturating_sub(window_size)
    } else {
        current.saturating_sub(sibling_count)
    };
    let end = (start + window_size - 1).min(total - 1);

    let mut pages = std::collections::BTreeSet::new();
    pages.insert(0);
    pages.insert(total - 1);
    for page in start..=end {
        pages.insert(page);
    }

    let mut items = Vec::new();
    let mut last = None;
    for page in pages {
        if let Some(prev) = last
            && page > prev + 1
        {
            if page == prev + 2 {
                items.push(PageItem::Page(prev + 1));
            } else {
                items.push(PageItem::Ellipsis);
            }
        }
        items.push(PageItem::Page(page));
        last = Some(page);
    }
    items
}

/// Left-pointing chevron icon.
fn chevron_left(class: &'static str) -> impl IntoView {
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <polyline points="15 18 9 12 15 6" />
        </svg>
    }
}

/// Right-pointing chevron icon.
fn chevron_right(class: &'static str) -> impl IntoView {
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <polyline points="9 18 15 12 9 6" />
        </svg>
    }
}

/// Single numbered page button used inside the pagination bar.
#[component]
fn PageButton(
    n: usize,
    #[prop(into)] active: Signal<bool>,
    set_page: WriteSignal<usize>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=move || {
                if active.get() {
                    "join-item btn btn-active".to_string()
                } else {
                    "join-item btn".to_string()
                }
            }
            aria-label=format!("Page {}", n + 1)
            aria-current=move || if active.get() { "page" } else { "" }
            on:click=move |_| set_page.set(n)
        >
            {n + 1}
        </button>
    }
}

/// DaisyUI-styled pagination bar with previous/next arrows and ellipsis gaps.
#[component]
pub fn Pagination(
    #[prop(into)] page: Signal<usize>,
    set_page: WriteSignal<usize>,
    #[prop(into)] total_pages: Signal<usize>,
    #[prop(default = DEFAULT_SIBLING_COUNT)] sibling_count: usize,
) -> impl IntoView {
    let display_page = Signal::derive(move || {
        let total = total_pages.get();
        page.get().min(total.saturating_sub(1))
    });

    view! {
        <div class="join" role="group" aria-label="Pagination">
            <button
                type="button"
                class="join-item btn"
                disabled=move || display_page.get() == 0
                aria-label="Previous page"
                on:click=move |_| set_page.update(|p| *p = p.saturating_sub(1))
            >
                {chevron_left("w-4 h-4")}
            </button>
            {move || {
                let current = display_page.get();
                let total = total_pages.get();
                pagination_pages(current, total, sibling_count)
                    .into_iter()
                    .map(|item| match item {
                        PageItem::Page(n) => {
                            let active = Signal::derive(move || display_page.get() == n);
                            view! {
                                <PageButton n=n active=active set_page=set_page />
                            }
                                .into_any()
                        }
                        PageItem::Ellipsis => view! {
                            <button
                                type="button"
                                class="join-item btn btn-disabled"
                                aria-hidden="true"
                            >
                                "..."
                            </button>
                        }
                            .into_any(),
                    })
                    .collect_view()
                    .into_any()
            }}
            <button
                type="button"
                class="join-item btn"
                disabled=move || display_page.get() + 1 >= total_pages.get()
                aria-label="Next page"
                on:click=move |_| set_page.update(|p| {
                    let max = total_pages.get().saturating_sub(1);
                    *p = (*p + 1).min(max)
                })
            >
                {chevron_right("w-4 h-4")}
            </button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_numbers(items: &[PageItem]) -> Vec<usize> {
        items
            .iter()
            .filter_map(|item| match item {
                PageItem::Page(n) => Some(*n),
                PageItem::Ellipsis => None,
            })
            .collect()
    }

    fn ellipsis_count(items: &[PageItem]) -> usize {
        items
            .iter()
            .filter(|item| matches!(item, PageItem::Ellipsis))
            .count()
    }

    #[test]
    fn empty_pagination_when_total_is_zero() {
        let items = pagination_pages(0, 0, 1);
        assert!(items.is_empty());
    }

    #[test]
    fn single_page_has_no_ellipsis() {
        let items = pagination_pages(0, 1, 1);
        assert_eq!(page_numbers(&items), vec![0]);
        assert_eq!(ellipsis_count(&items), 0);
    }

    #[test]
    fn all_pages_visible_when_total_is_small() {
        let items = pagination_pages(1, 5, 1);
        assert_eq!(page_numbers(&items), vec![0, 1, 2, 3, 4]);
        assert_eq!(ellipsis_count(&items), 0);
    }

    #[test]
    fn first_page_shows_head_and_truncated_tail() {
        let items = pagination_pages(0, 20, 1);
        assert_eq!(page_numbers(&items), vec![0, 1, 2, 19]);
        assert_eq!(ellipsis_count(&items), 1);
    }

    #[test]
    fn middle_page_shows_both_ellipsis_gaps() {
        let items = pagination_pages(9, 20, 1);
        assert_eq!(page_numbers(&items), vec![0, 8, 9, 10, 19]);
        assert_eq!(ellipsis_count(&items), 2);
    }

    #[test]
    fn last_page_shows_truncated_head() {
        let items = pagination_pages(19, 20, 1);
        assert_eq!(page_numbers(&items), vec![0, 17, 18, 19]);
        assert_eq!(ellipsis_count(&items), 1);
    }

    #[test]
    fn larger_sibling_window_includes_more_neighbors() {
        let items = pagination_pages(9, 20, 2);
        assert_eq!(page_numbers(&items), vec![0, 7, 8, 9, 10, 11, 19]);
        assert_eq!(ellipsis_count(&items), 2);
    }
}
