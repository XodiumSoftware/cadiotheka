//! 3D IFC model viewer using the `three-d` renderer.
//!
//! The viewer fetches a pre-converted GLB from the backend (`/data/projects/:id/glb`)
//! and renders it with the `three-d` renderer in [`crate::three_d_viewer`].

use crate::components::Icon;
use crate::components::ui::view_gizmo::{GizmoPosition, ViewGizmo, ViewGizmoDirection};
use crate::three_d_viewer::{OrbitControls, Renderer, ViewState, ViewerSettings, ViewerTheme};
use crate::utils::{local_storage_get, local_storage_remove, local_storage_set};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

/// Viewer states exposed to the parent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IfcViewerState {
    /// No model URL provided.
    NoModel,
    /// Downloading the GLB bytes.
    Loading,
    /// Parsing the GLB geometry.
    Processing,
    /// Rendering the model.
    Rendering,
    /// The model could not be loaded or parsed.
    Error,
}

/// Renders an IFC model from the given URL into a canvas.
///
/// Optional signals let a parent read viewer state and control debug
/// visibility. When omitted, internal signals are used.
#[component]
pub fn IfcViewer(
    #[prop(into)] url: Signal<Option<String>>,
    #[prop(into, optional)] storage_key: Option<Signal<String>>,
    #[prop(optional)] state_signal: Option<RwSignal<IfcViewerState>>,
    #[prop(optional)] reset_view_signal: Option<RwSignal<bool>>,
    #[prop(optional)] show_axes_signal: Option<RwSignal<bool>>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] show_gizmo_signal: Option<RwSignal<bool>>,
    #[prop(into, optional)] gizmo_position_signal: Option<RwSignal<GizmoPosition>>,
    #[prop(into, optional)] gizmo_edit_mode_signal: Option<RwSignal<bool>>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let state = state_signal.unwrap_or_else(|| RwSignal::new(IfcViewerState::NoModel));
    let reset_view = reset_view_signal.unwrap_or_else(|| RwSignal::new(false));
    let show_axes = show_axes_signal.unwrap_or_else(|| RwSignal::new(true));
    let disabled = disabled.unwrap_or_else(|| Signal::derive(|| false));
    let show_gizmo = show_gizmo_signal.unwrap_or_else(|| RwSignal::new(true));
    let gizmo_position =
        gizmo_position_signal.unwrap_or_else(|| RwSignal::new(GizmoPosition::TopRight));
    let gizmo_edit_mode = gizmo_edit_mode_signal.unwrap_or_else(|| RwSignal::new(false));

    let focus_direction: RwSignal<Option<ViewGizmoDirection>> = RwSignal::new(None);

    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::new(RefCell::new(None));
    let controls: Rc<RefCell<OrbitControls>> = Rc::new(RefCell::new(OrbitControls::default()));
    let dirty = Rc::new(RefCell::new(false));
    let pending_frame = Rc::new(RefCell::new(false));
    let animation_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    let save_generation: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

    let schedule_save = {
        let renderer = Rc::clone(&renderer);
        let save_generation = Rc::clone(&save_generation);
        move || {
            let Some(key) = storage_key
                .as_ref()
                .map(|s| s.get())
                .filter(|k| !k.is_empty())
            else {
                return;
            };
            let state_json = renderer
                .borrow()
                .as_ref()
                .map(|r| r.save_view_state().to_json())
                .unwrap_or_default();
            if state_json.is_empty() {
                return;
            }

            let expected = {
                let mut generation = save_generation.borrow_mut();
                *generation = generation.wrapping_add(1);
                *generation
            };

            leptos::task::spawn_local(async move {
                TimeoutFuture::new(500).await;
                if *save_generation.borrow() != expected {
                    return;
                }
                local_storage_set(&key, &state_json);
            });
        }
    };

    let settings_generation: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

    let schedule_save_settings = {
        let settings_generation = Rc::clone(&settings_generation);
        move || {
            let Some(key) = storage_key
                .as_ref()
                .map(|s| s.get())
                .filter(|k| !k.is_empty())
            else {
                return;
            };
            let settings = ViewerSettings {
                show_axes: show_axes.get(),
                show_gizmo: show_gizmo.get(),
            };
            let settings_json = settings.to_json();
            let settings_key = format!("{key}.settings");

            let expected = {
                let mut generation = settings_generation.borrow_mut();
                *generation = generation.wrapping_add(1);
                *generation
            };

            let generation = Rc::clone(&settings_generation);
            leptos::task::spawn_local(async move {
                TimeoutFuture::new(500).await;
                if *generation.borrow() != expected {
                    return;
                }
                local_storage_set(&settings_key, &settings_json);
            });
        }
    };

    let request_render: Rc<RefCell<dyn FnMut()>> = {
        let schedule_save = schedule_save.clone();
        let renderer = Rc::clone(&renderer);
        let dirty = Rc::clone(&dirty);
        let pending_frame = Rc::clone(&pending_frame);
        let animation_handle = Rc::clone(&animation_handle);
        Rc::new(RefCell::new(move || {
            *dirty.borrow_mut() = true;
            if disabled.get() {
                return;
            }
            if *pending_frame.borrow() {
                return;
            }
            *pending_frame.borrow_mut() = true;

            let closure = {
                let renderer = Rc::clone(&renderer);
                let dirty = Rc::clone(&dirty);
                let pending_frame = Rc::clone(&pending_frame);
                let animation_handle = Rc::clone(&animation_handle);
                let schedule_save = schedule_save.clone();
                let disabled = disabled;
                Closure::<dyn FnMut()>::new(move || {
                    *pending_frame.borrow_mut() = false;
                    *animation_handle.borrow_mut() = None;
                    if *dirty.borrow() && !disabled.get() {
                        if let Some(renderer) = renderer.borrow_mut().as_mut() {
                            renderer.render();
                        }
                        *dirty.borrow_mut() = false;
                        let schedule_save = schedule_save.clone();
                        schedule_save();
                    }
                })
            };

            let Some(window) = leptos::web_sys::window() else {
                *pending_frame.borrow_mut() = false;
                return;
            };
            if let Ok(handle) = window.request_animation_frame(closure.as_ref().unchecked_ref()) {
                *animation_handle.borrow_mut() = Some(handle);
                closure.forget();
            } else {
                *pending_frame.borrow_mut() = false;
            }
        }))
    };

    on_cleanup({
        let animation_handle = SendWrapper::new(Rc::clone(&animation_handle));
        let renderer = SendWrapper::new(Rc::clone(&renderer));
        move || {
            if let Some(handle) = (*animation_handle).borrow_mut().take()
                && let Some(window) = leptos::web_sys::window()
            {
                let _ = window.cancel_animation_frame(handle);
            }
            *(*renderer).borrow_mut() = None;
        }
    });

    Effect::new({
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        move |_| {
            let Some(dir) = focus_direction.get() else {
                return;
            };
            focus_direction.set(None);
            let has_renderer = {
                let mut renderer_ref = renderer.borrow_mut();
                if let Some(renderer) = renderer_ref.as_mut() {
                    renderer.set_focus(dir.into());
                    true
                } else {
                    false
                }
            };
            if has_renderer {
                request_render.borrow_mut()();
            }
        }
    });

    Effect::new({
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        move |_| {
            if !reset_view.get() {
                return;
            }
            reset_view.set(false);

            if let Some(key) = storage_key
                .as_ref()
                .map(|s| s.get())
                .filter(|k| !k.is_empty())
            {
                local_storage_remove(&key);
            }

            let has_renderer = {
                let mut renderer_ref = renderer.borrow_mut();
                if let Some(renderer) = renderer_ref.as_mut() {
                    renderer.reset_view(show_axes.get());
                    true
                } else {
                    false
                }
            };
            if has_renderer {
                request_render.borrow_mut()();
            }
        }
    });

    Effect::new({
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        move |_| {
            let has_renderer = {
                let mut renderer_ref = renderer.borrow_mut();
                if let Some(renderer) = renderer_ref.as_mut() {
                    renderer.set_theme(ViewerTheme::Dark);
                    true
                } else {
                    false
                }
            };
            if has_renderer {
                request_render.borrow_mut()();
            }
        }
    });

    Effect::new({
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        let schedule_save_settings = schedule_save_settings.clone();
        move |_| {
            let show = show_axes.get();
            {
                let mut renderer_ref = renderer.borrow_mut();
                if let Some(renderer) = renderer_ref.as_mut() {
                    renderer.set_show_axes(show);
                }
            }
            request_render.borrow_mut()();
            schedule_save_settings();
        }
    });

    Effect::new({
        let request_render = Rc::clone(&request_render);
        let schedule_save_settings = schedule_save_settings.clone();
        move |_| {
            let _ = show_gizmo.get();
            request_render.borrow_mut()();
            schedule_save_settings();
        }
    });

    Effect::new({
        move |_| {
            let Some(key) = storage_key
                .as_ref()
                .map(|s| s.get())
                .filter(|k| !k.is_empty())
            else {
                return;
            };
            let Some(settings) = load_viewer_settings(&format!("{key}.settings")) else {
                return;
            };
            show_axes.set(settings.show_axes);
            show_gizmo.set(settings.show_gizmo);
        }
    });

    let on_mouse_down = {
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        let controls = Rc::clone(&controls);
        move |ev: leptos::web_sys::MouseEvent| {
            let mut state = controls.borrow_mut();
            if state.on_mouse_down(&ev, &renderer, show_axes.get()) {
                request_render.borrow_mut()();
            }
        }
    };
    let on_mouse_move = {
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        let controls = Rc::clone(&controls);
        move |ev: leptos::web_sys::MouseEvent| {
            let state = controls.borrow();
            if state.on_mouse_move(&ev, &renderer) {
                request_render.borrow_mut()();
            }
        }
    };
    let on_mouse_up = {
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        let controls = Rc::clone(&controls);
        move |ev: leptos::web_sys::MouseEvent| {
            let mut state = controls.borrow_mut();
            if state.on_mouse_up(&ev, &renderer) {
                request_render.borrow_mut()();
            }
        }
    };
    let on_wheel = {
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        move |ev: leptos::web_sys::WheelEvent| {
            if OrbitControls::on_wheel(&ev, &renderer) {
                request_render.borrow_mut()();
            }
        }
    };
    let on_context_menu = |ev: leptos::web_sys::MouseEvent| {
        ev.prevent_default();
    };
    let on_mouse_leave = {
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        let controls = Rc::clone(&controls);
        move |_: leptos::web_sys::MouseEvent| {
            let mut state = controls.borrow_mut();
            state.on_mouse_leave(&renderer);
            request_render.borrow_mut()();
        }
    };

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(url) = url.get() else {
            state.set(IfcViewerState::NoModel);
            return;
        };

        if renderer.borrow().is_none() {
            if let Some(new_renderer) = Renderer::new(&canvas) {
                *renderer.borrow_mut() = Some(new_renderer);
            } else {
                state.set(IfcViewerState::Error);
                return;
            }
        }

        state.set(IfcViewerState::Loading);
        let renderer = Rc::clone(&renderer);
        let state = state;
        let request_render = Rc::clone(&request_render);

        leptos::task::spawn_local(async move {
            match load_model_bytes(&url).await {
                Some(glb_bytes) => {
                    state.set(IfcViewerState::Processing);

                    let load_ok = renderer
                        .borrow_mut()
                        .as_mut()
                        .is_some_and(|r| r.load_model(&glb_bytes));

                    if load_ok {
                        state.set(IfcViewerState::Rendering);

                        let restored = storage_key
                            .as_ref()
                            .map(|s| s.get())
                            .filter(|k| !k.is_empty())
                            .and_then(|key| load_view_state(&key));

                        if let Some(state) = restored {
                            if let Some(r) = renderer.borrow_mut().as_mut() {
                                r.restore_view_state(&state, show_axes.get());
                            }
                        } else if let Some(r) = renderer.borrow_mut().as_mut() {
                            r.reset_view(show_axes.get());
                        }

                        if let Some(r) = renderer.borrow_mut().as_mut() {
                            r.set_show_axes(show_axes.get());
                        }

                        request_render.borrow_mut()();
                    } else {
                        state.set(IfcViewerState::Error);
                    }
                }
                None => {
                    state.set(IfcViewerState::Error);
                }
            }
        });
    });

    view! {
        <div class="relative w-full h-full overflow-hidden border border-base-content/10">
            <canvas
                node_ref=canvas_ref
                class=move || {
                    if disabled.get() {
                        "w-full h-full block cursor-grab active:cursor-grabbing hidden".to_string()
                    } else {
                        "w-full h-full block cursor-grab active:cursor-grabbing".to_string()
                    }
                }
                aria-label="IFC 3D viewer"
                on:mousedown=on_mouse_down
                on:mousemove=on_mouse_move
                on:mouseup=on_mouse_up
                on:mouseleave=on_mouse_leave
                on:wheel=on_wheel
                on:contextmenu=on_context_menu
            />
            {move || if disabled.get() {
                view! {
                    <div class="absolute inset-0 flex flex-col items-center justify-center gap-2 text-base-content/50 text-sm pointer-events-none bg-base-100/80 z-10">
                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                        <span>"3D viewer is off while editing."</span>
                    </div>
                }.into_any()
            } else {
                match state.get() {
                    IfcViewerState::NoModel => view! {
                        <div class="absolute inset-0 flex items-center justify-center text-base-content/50 text-sm pointer-events-none">
                            "No IFC model uploaded yet."
                        </div>
                    }.into_any(),
                    IfcViewerState::Loading => view! {
                        <div class="absolute inset-0 flex items-center justify-center text-base-content/50 text-sm pointer-events-none gap-2">
                            <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                            <span>"Loading model..."</span>
                        </div>
                    }.into_any(),
                    IfcViewerState::Processing => view! {
                        <div class="absolute inset-0 flex items-center justify-center text-base-content/50 text-sm pointer-events-none gap-2">
                            <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                            <span>"Processing geometry..."</span>
                        </div>
                    }.into_any(),
                    IfcViewerState::Error => view! {
                        <div class="absolute inset-0 flex items-center justify-center text-error text-sm pointer-events-none">
                            "Failed to load IFC model."
                        </div>
                    }.into_any(),
                    IfcViewerState::Rendering => view! {
                        <div class=move || if gizmo_edit_mode.get() { "absolute inset-0 z-20 pointer-events-none".to_string() } else { "absolute inset-0 z-20".to_string() }>
                            {move || if show_gizmo.get() {
                                view! {
                                    <ViewGizmo
                                        disabled=Signal::derive({
                                            let disabled = disabled;
                                            move || disabled.get() || state.get() != IfcViewerState::Rendering
                                        })
                                        position=Signal::derive(move || gizmo_position.get())
                                        editing=Signal::derive(move || gizmo_edit_mode.get())
                                        on_direction=Callback::new(move |dir| {
                                            focus_direction.set(Some(dir));
                                        })
                                    />
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                            {move || if gizmo_edit_mode.get() {
                                view! {
                                    <GizmoPositionSelector
                                        current=Signal::derive(move || gizmo_position.get())
                                        on_select=Callback::new(move |pos| {
                                            gizmo_position.set(pos);
                                        })
                                    />
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                            {move || if gizmo_edit_mode.get() {
                                view! {
                                    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                        <span class="text-xs text-error bg-base-100/80 px-2 py-1 border border-error/30 shadow backdrop-blur-sm">
                                            "R+click the view gizmo button again to close edit mode."
                                        </span>
                                    </div>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}

async fn load_model_bytes(url: &str) -> Option<Vec<u8>> {
    match Request::get(url).send().await {
        Ok(response) if response.ok() => response.binary().await.ok(),
        Ok(response) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch model: HTTP {}", response.status()).into(),
            );
            None
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to fetch model: {err:?}").into());
            None
        }
    }
}

fn load_view_state(key: &str) -> Option<ViewState> {
    let json = local_storage_get(key)?;
    ViewState::from_json(&json)
}

fn load_viewer_settings(key: &str) -> Option<ViewerSettings> {
    let json = local_storage_get(key)?;
    ViewerSettings::from_json(&json)
}

/// Overlay that renders red arrow buttons on each edge and corner so the user
/// can pick a new gizmo position while in edit mode.
#[component]
fn GizmoPositionSelector(
    #[prop(into)] current: Signal<GizmoPosition>,
    #[prop(into)] on_select: Callback<GizmoPosition>,
) -> impl IntoView {
    view! {
        <div class="absolute inset-0 pointer-events-none">
            <GizmoPositionButton
                pos=GizmoPosition::TopLeft
                class="top-3 left-3"
                current=current
                on_select=on_select
            >
                <Icon::ArrowUpLeft class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::TopCenter
                class="top-3 left-1/2 -translate-x-1/2"
                current=current
                on_select=on_select
            >
                <Icon::ArrowUp class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::TopRight
                class="top-3 right-3"
                current=current
                on_select=on_select
            >
                <Icon::ArrowUpRight class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::LeftCenter
                class="left-3 top-1/2 -translate-y-1/2"
                current=current
                on_select=on_select
            >
                <Icon::ArrowLeft class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::RightCenter
                class="right-3 top-1/2 -translate-y-1/2"
                current=current
                on_select=on_select
            >
                <Icon::ArrowRight class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::BottomLeft
                class="bottom-3 left-3"
                current=current
                on_select=on_select
            >
                <Icon::ArrowDownLeft class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::BottomCenter
                class="bottom-3 left-1/2 -translate-x-1/2"
                current=current
                on_select=on_select
            >
                <Icon::ArrowDown class="w-6 h-6"/>
            </GizmoPositionButton>
            <GizmoPositionButton
                pos=GizmoPosition::BottomRight
                class="bottom-3 right-3"
                current=current
                on_select=on_select
            >
                <Icon::ArrowDownRight class="w-6 h-6"/>
            </GizmoPositionButton>
        </div>
    }
}

/// A single arrow button used by [`GizmoPositionSelector`].
#[component]
fn GizmoPositionButton(
    #[prop(into)] pos: GizmoPosition,
    class: &'static str,
    #[prop(into)] current: Signal<GizmoPosition>,
    #[prop(into)] on_select: Callback<GizmoPosition>,
    children: Children,
) -> impl IntoView {
    let is_current = move || current.get() == pos;
    view! {
        <button
            type="button"
            class=move || format!("absolute {class} pointer-events-auto btn btn-ghost btn-xs p-1.5 h-auto min-h-0 border-2 border-transparent {}", if is_current() { "text-error border-error bg-base-100/90" } else { "text-error/70 hover:text-error hover:border-error" })
            aria-label=move || format!("Move view gizmo to the {}", pos.label())
            on:click=move |_| on_select.run(pos)
        >
            {children()}
        </button>
    }
}

impl GizmoPosition {
    fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "top left",
            Self::TopCenter => "top center",
            Self::TopRight => "top right",
            Self::LeftCenter => "left center",
            Self::RightCenter => "right center",
            Self::BottomLeft => "bottom left",
            Self::BottomCenter => "bottom center",
            Self::BottomRight => "bottom right",
        }
    }
}
