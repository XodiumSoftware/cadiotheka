//! Small overlay gizmo that snaps the 3D viewer camera to fixed directions.

use crate::three_d_viewer::ViewDirection;
use leptos::prelude::*;

/// Callback wrapper used by [`ViewGizmo`] so callers do not need to expose the
/// internal [`ViewDirection`] type directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewGizmoDirection {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl From<ViewGizmoDirection> for ViewDirection {
    fn from(value: ViewGizmoDirection) -> Self {
        match value {
            ViewGizmoDirection::Front => Self::Front,
            ViewGizmoDirection::Back => Self::Back,
            ViewGizmoDirection::Left => Self::Left,
            ViewGizmoDirection::Right => Self::Right,
            ViewGizmoDirection::Top => Self::Top,
            ViewGizmoDirection::Bottom => Self::Bottom,
        }
    }
}

impl ViewGizmoDirection {
    fn label(self) -> &'static str {
        ViewDirection::from(self).label()
    }
}

/// Renders a compact cube-like gizmo with one button per face.
#[component]
pub fn ViewGizmo(
    #[prop(into)] on_direction: Callback<ViewGizmoDirection>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or_else(|| Signal::derive(|| false));

    let directions = [
        (
            ViewGizmoDirection::Top,
            "top-0 left-1/2 -translate-x-1/2",
            "T",
        ),
        (
            ViewGizmoDirection::Bottom,
            "bottom-0 left-1/2 -translate-x-1/2",
            "B",
        ),
        (
            ViewGizmoDirection::Front,
            "top-1/2 right-0 -translate-y-1/2",
            "F",
        ),
        (
            ViewGizmoDirection::Back,
            "top-1/2 left-0 -translate-y-1/2",
            "K",
        ),
        (ViewGizmoDirection::Left, "top-0 left-0", "L"),
        (ViewGizmoDirection::Right, "bottom-0 right-0", "R"),
    ];

    view! {
        <div class="pointer-events-auto select-none rounded-lg border border-base-content/10 bg-base-100/80 p-1 shadow backdrop-blur-sm">
            <div class="relative h-20 w-20">
                {directions.into_iter().map(|(dir, class, text)| {
                    let label = dir.label();
                    view! {
                        <button
                            type="button"
                            class=format!(
                                "absolute h-7 w-7 btn btn-xs btn-square min-h-0 {class}"
                            )
                            aria-label=format!("View {label}")
                            disabled=move || disabled.get()
                            on:click=move |_| on_direction.run(dir)
                        >
                            <span class="text-xs font-semibold">{text}</span>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
