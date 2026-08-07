//! Circular overlay gizmo that snaps the 3D viewer camera to fixed directions.
//!
//! The gizmo is split into eight outer ring slices for the cardinal and
//! inter-cardinal directions, plus an inner circle split horizontally into Top
//! and Bottom semicircles.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::three_d_viewer::ViewDirection;
use leptos::prelude::*;

/// Direction slice exposed by [`ViewGizmo`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewGizmoDirection {
    Top,
    Bottom,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl From<ViewGizmoDirection> for ViewDirection {
    fn from(value: ViewGizmoDirection) -> Self {
        match value {
            ViewGizmoDirection::Top => Self::Top,
            ViewGizmoDirection::Bottom => Self::Bottom,
            ViewGizmoDirection::North => Self::Back,
            ViewGizmoDirection::NorthEast => Self::BackRight,
            ViewGizmoDirection::East => Self::Right,
            ViewGizmoDirection::SouthEast => Self::FrontRight,
            ViewGizmoDirection::South => Self::Front,
            ViewGizmoDirection::SouthWest => Self::FrontLeft,
            ViewGizmoDirection::West => Self::Left,
            ViewGizmoDirection::NorthWest => Self::BackLeft,
        }
    }
}

impl ViewGizmoDirection {
    fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::North => "N",
            Self::NorthEast => "NE",
            Self::East => "E",
            Self::SouthEast => "SE",
            Self::South => "S",
            Self::SouthWest => "SW",
            Self::West => "W",
            Self::NorthWest => "NW",
        }
    }

    fn aria_label(self) -> &'static str {
        match self {
            Self::Top => "View from top",
            Self::Bottom => "View from bottom",
            Self::North => "View from north",
            Self::NorthEast => "View from north-east",
            Self::East => "View from east",
            Self::SouthEast => "View from south-east",
            Self::South => "View from south",
            Self::SouthWest => "View from south-west",
            Self::West => "View from west",
            Self::NorthWest => "View from north-west",
        }
    }
}

const RADIUS: u32 = 60;
const INNER_RADIUS: u32 = 40;
const CENTER: u32 = RADIUS;
const DIAMETER: u32 = RADIUS * 2;

/// Renders a circular view-direction gizmo.
#[component]
pub fn ViewGizmo(
    #[prop(into)] on_direction: Callback<ViewGizmoDirection>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    let _disabled = disabled.unwrap_or_else(|| Signal::derive(|| false));
    let diameter = DIAMETER;
    let inner_radius = INNER_RADIUS;
    let center = CENTER;

    let slices = [
        (ViewGizmoDirection::North, 247.5, 292.5),
        (ViewGizmoDirection::NorthEast, 292.5, 337.5),
        (ViewGizmoDirection::East, 337.5, 22.5),
        (ViewGizmoDirection::SouthEast, 22.5, 67.5),
        (ViewGizmoDirection::South, 67.5, 112.5),
        (ViewGizmoDirection::SouthWest, 112.5, 157.5),
        (ViewGizmoDirection::West, 157.5, 202.5),
        (ViewGizmoDirection::NorthWest, 202.5, 247.5),
    ];

    view! {
        <div class="pointer-events-auto select-none rounded-full border border-base-content/10 bg-base-100/90 shadow backdrop-blur-sm p-1">
            <svg
                width=diameter
                height=diameter
                viewBox=format!("0 0 {diameter} {diameter}")
                aria-label="Camera direction gizmo"
                class="block"
            >
                {slices.into_iter().map(|(dir, start, end)| {
                    let path = annular_slice_path(center, center, RADIUS, INNER_RADIUS, start, end);
                    let label = dir.label();
                    let aria = dir.aria_label();
                    let mid = mid_angle(start, end);
                    let (tx, ty) = label_position(center, u32::midpoint(RADIUS, INNER_RADIUS), mid);
                    view! {
                        <g
                            class="group cursor-pointer"
                            on:click=move |_| on_direction.run(dir)
                        >
                            <path
                                d=path
                                fill="currentColor"
                                class="fill-base-content text-[11px] font-bold group-hover:fill-black"
                                aria-label=aria
                            />
                            <text
                                x=tx
                                y=ty
                                text-anchor="middle"
                                dominant-baseline="central"
                                class="fill-base-content text-[11px] font-bold pointer-events-none group-hover:fill-black"
                            >
                                {label}
                            </text>
                        </g>
                    }
                }).collect_view()}

                <g class="cursor-pointer group" on:click=move |_| on_direction.run(ViewGizmoDirection::Top)>
                    <path
                        d=upper_semicircle_path(center, center, INNER_RADIUS)
                        fill="currentColor"
                        class="text-base-content/10 hover:text-primary transition-colors"
                        aria-label="View from top"
                    />
                    <text
                        x=center
                        y=center - INNER_RADIUS / 2
                        text-anchor="middle"
                        dominant-baseline="central"
                        class="fill-base-content text-[11px] font-bold pointer-events-none group-hover:fill-black"
                    >
                        "T"
                    </text>
                </g>
                <g class="cursor-pointer group" on:click=move |_| on_direction.run(ViewGizmoDirection::Bottom)>
                    <path
                        d=lower_semicircle_path(center, center, INNER_RADIUS)
                        fill="currentColor"
                        class="text-base-content/10 hover:text-primary transition-colors"
                        aria-label="View from bottom"
                    />
                    <text
                        x=center
                        y=center + INNER_RADIUS / 2
                        text-anchor="middle"
                        dominant-baseline="central"
                        class="fill-base-content text-[11px] font-bold pointer-events-none group-hover:fill-black"
                    >
                        "B"
                    </text>
                </g>

                <circle
                    cx=center
                    cy=center
                    r=inner_radius
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1"
                    class="text-base-content/30"
                />
                <line
                    x1=inner_radius
                    y1=center
                    x2=diameter - inner_radius
                    y2=center
                    stroke="currentColor"
                    stroke-width="1"
                    class="text-base-content/30"
                />
            </svg>
        </div>
    }
}

fn polar_point(cx: u32, cy: u32, radius: u32, angle_deg: f32) -> (f32, f32) {
    let rad = angle_deg.to_radians();
    let x = cx as f32 + radius as f32 * rad.cos();
    let y = cy as f32 + radius as f32 * rad.sin();
    (x, y)
}

fn mid_angle(start: f32, end: f32) -> f32 {
    let a = start;
    let mut b = end;
    if b < a {
        b += 360.0;
    }
    let mut mid = f32::midpoint(a, b);
    if mid >= 360.0 {
        mid -= 360.0;
    }
    mid
}

fn label_position(cx: u32, radius: u32, angle_deg: f32) -> (f32, f32) {
    let (x, y) = polar_point(cx, cx, radius, angle_deg);
    (x, y)
}

fn annular_slice_path(
    cx: u32,
    cy: u32,
    outer_r: u32,
    inner_r: u32,
    start_deg: f32,
    end_deg: f32,
) -> String {
    let (x1, y1) = polar_point(cx, cy, outer_r, start_deg);
    let (x2, y2) = polar_point(cx, cy, outer_r, end_deg);
    let (x3, y3) = polar_point(cx, cy, inner_r, end_deg);
    let (x4, y4) = polar_point(cx, cy, inner_r, start_deg);

    let large_arc = i32::from(sweep_angle(start_deg, end_deg) > 180.0);

    format!(
        "M {x1:.1} {y1:.1} A {outer_r} {outer_r} 0 {large_arc} 1 {x2:.1} {y2:.1} L {x3:.1} {y3:.1} A {inner_r} {inner_r} 0 {large_arc} 0 {x4:.1} {y4:.1} Z"
    )
}

fn sweep_angle(start: f32, end: f32) -> f32 {
    let mut b = end;
    if b < start {
        b += 360.0;
    }
    b - start
}

fn upper_semicircle_path(cx: u32, cy: u32, r: u32) -> String {
    let (lx, ly) = polar_point(cx, cy, r, 180.0);
    let (rx, ry) = polar_point(cx, cy, r, 0.0);
    format!("M {cx} {cy} L {lx:.1} {ly:.1} A {r} {r} 0 0 1 {rx:.1} {ry:.1} Z")
}

fn lower_semicircle_path(cx: u32, cy: u32, r: u32) -> String {
    let (lx, ly) = polar_point(cx, cy, r, 180.0);
    let (rx, ry) = polar_point(cx, cy, r, 0.0);
    format!("M {cx} {cy} L {rx:.1} {ry:.1} A {r} {r} 0 0 1 {lx:.1} {ly:.1} Z")
}
