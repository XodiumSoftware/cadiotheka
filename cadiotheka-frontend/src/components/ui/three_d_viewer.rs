//! 3D IFC model viewer using the `three-d` renderer.
//!
//! The viewer fetches a pre-converted GLB from the backend (`/data/projects/:id/glb`)
//! and renders it with the `three-d` renderer in [`crate::three_d_viewer`].

use crate::three_d_viewer::{OrbitControls, Renderer, ViewState, ViewerTheme};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::fmt::Write;
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
    #[prop(optional)] fps_signal: Option<RwSignal<f64>>,
    #[prop(optional)] show_debug_signal: Option<RwSignal<bool>>,
    #[prop(optional)] debug_text_signal: Option<RwSignal<String>>,
    #[prop(optional)] reset_view_signal: Option<RwSignal<bool>>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let state = state_signal.unwrap_or_else(|| RwSignal::new(IfcViewerState::NoModel));
    let fps = fps_signal.unwrap_or_else(|| RwSignal::new(0.0_f64));
    let show_debug = show_debug_signal.unwrap_or_else(|| RwSignal::new(false));
    let debug_text = debug_text_signal.unwrap_or_else(|| RwSignal::new(String::new()));
    let reset_view = reset_view_signal.unwrap_or_else(|| RwSignal::new(false));

    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::new(RefCell::new(None));
    let controls: Rc<RefCell<Option<OrbitControls>>> = Rc::new(RefCell::new(None));
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
                if let Some(window) = leptos::web_sys::window()
                    && let Ok(Some(storage)) = window.local_storage()
                {
                    let _ = storage.set_item(&key, &state_json);
                }
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
                Closure::<dyn FnMut()>::new(move || {
                    *pending_frame.borrow_mut() = false;
                    *animation_handle.borrow_mut() = None;
                    if *dirty.borrow() {
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

    let update_debug = {
        let renderer = Rc::clone(&renderer);
        let last_time = RefCell::new(0.0_f64);
        let frame_count = RefCell::new(0_u32);
        let last_fps_update = RefCell::new(0.0_f64);
        move || {
            let renderer_ref = renderer.borrow();
            let Some(renderer) = renderer_ref.as_ref() else {
                return;
            };
            let camera = renderer.camera();
            let (min, max) = renderer.scene_bounds();
            let eye = camera.position();
            let target = camera.target();
            let dx = eye.x - target.x;
            let dy = eye.y - target.y;
            let dz = eye.z - target.z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            let mut text = String::new();
            let _ = writeln!(text, "eye: [{:.2}, {:.2}, {:.2}]", eye.x, eye.y, eye.z);
            let _ = writeln!(
                text,
                "target: [{:.2}, {:.2}, {:.2}]",
                target.x, target.y, target.z
            );
            let _ = writeln!(text, "distance: {distance:.2}");
            let _ = writeln!(
                text,
                "near: {:.3}, far: {:.1}",
                camera.z_near(),
                camera.z_far()
            );
            let _ = writeln!(
                text,
                "bounds min: [{:.2}, {:.2}, {:.2}]",
                min[0], min[1], min[2]
            );
            let _ = writeln!(
                text,
                "bounds max: [{:.2}, {:.2}, {:.2}]",
                max[0], max[1], max[2]
            );
            let _ = writeln!(
                text,
                "size: [{:.2}, {:.2}, {:.2}]",
                max[0] - min[0],
                max[1] - min[1],
                max[2] - min[2]
            );
            let _ = writeln!(text, "primitives: {}", renderer.primitive_count());
            let _ = writeln!(text, "vertices: {}", renderer.total_vertices());
            let _ = writeln!(text, "triangles: {}", renderer.total_triangles());
            debug_text.set(text);

            let window = leptos::web_sys::window().and_then(|w| w.performance());
            if let Some(performance) = window {
                let now: f64 = performance.now();
                *frame_count.borrow_mut() += 1;
                if now - *last_fps_update.borrow() >= 500.0 {
                    let fps_value = f64::from(*frame_count.borrow()) * 1000.0
                        / (now - *last_fps_update.borrow());
                    fps.set(fps_value);
                    *last_fps_update.borrow_mut() = now;
                    *frame_count.borrow_mut() = 0;
                }
                *last_time.borrow_mut() = now;
            }
        }
    };

    on_cleanup({
        let animation_handle = SendWrapper::new(Rc::clone(&animation_handle));
        let renderer = SendWrapper::new(Rc::clone(&renderer));
        let controls = SendWrapper::new(Rc::clone(&controls));
        move || {
            if let Some(handle) = (*animation_handle).borrow_mut().take()
                && let Some(window) = leptos::web_sys::window()
            {
                let _ = window.cancel_animation_frame(handle);
            }
            *(*renderer).borrow_mut() = None;
            *(*controls).borrow_mut() = None;
        }
    });

    Effect::new({
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        let update_debug = update_debug.clone();
        move |_| {
            if !reset_view.get() {
                return;
            }
            reset_view.set(false);

            if let Some(key) = storage_key
                .as_ref()
                .map(|s| s.get())
                .filter(|k| !k.is_empty())
                && let Some(window) = leptos::web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
            {
                let _ = storage.remove_item(&key);
            }

            if let Some(renderer) = renderer.borrow_mut().as_mut() {
                renderer.reset_view();
                request_render.borrow_mut()();
                update_debug();
            }
        }
    });

    Effect::new({
        let renderer = Rc::clone(&renderer);
        let request_render = Rc::clone(&request_render);
        move |_| {
            if let Some(renderer) = renderer.borrow_mut().as_mut() {
                renderer.set_theme(ViewerTheme::Dark);
                request_render.borrow_mut()();
            }
        }
    });

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(url) = url.get() else {
            state.set(IfcViewerState::NoModel);
            return;
        };

        state.set(IfcViewerState::Loading);
        let renderer = Rc::clone(&renderer);
        let controls = Rc::clone(&controls);
        let state = state;
        let request_render = Rc::clone(&request_render);
        let update_debug = update_debug.clone();

        leptos::task::spawn_local(async move {
            match load_model_bytes(&url).await {
                Some(glb_bytes) => {
                    state.set(IfcViewerState::Processing);

                    if let Some(new_renderer) = Renderer::new(&canvas, &glb_bytes) {
                        *renderer.borrow_mut() = Some(new_renderer);
                        let render_callback = {
                            let request_render = Rc::clone(&request_render);
                            let update_debug = update_debug.clone();
                            move || {
                                request_render.borrow_mut()();
                                update_debug();
                            }
                        };
                        let new_controls = OrbitControls::attach(&renderer, render_callback);
                        *controls.borrow_mut() = Some(new_controls);
                        state.set(IfcViewerState::Rendering);

                        let restored = storage_key
                            .as_ref()
                            .map(|s| s.get())
                            .filter(|k| !k.is_empty())
                            .and_then(|key| load_view_state(&key));

                        if let Some(state) = restored {
                            if let Some(r) = renderer.borrow_mut().as_mut() {
                                r.restore_view_state(&state);
                            }
                        } else {
                            renderer.borrow_mut().as_mut().map(Renderer::reset_view);
                        }

                        request_render.borrow_mut()();
                        update_debug();
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
                class="w-full h-full block cursor-grab active:cursor-grabbing"
                aria-label="IFC 3D viewer"
            />
            {move || match state.get() {
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
                    {move || show_debug.get().then(|| view! {
                        <div class="absolute top-2 left-2 z-10 max-w-[20rem] bg-base-100/90 backdrop-blur text-xs font-mono p-3 rounded border border-base-content/10 text-base-content/80 whitespace-pre-wrap">
                            {move || debug_text.get()}
                        </div>
                    })}
                }.into_any(),
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
    let window = leptos::web_sys::window()?;
    let storage = window.local_storage().ok().flatten()?;
    let json = storage.get_item(key).ok().flatten()?;
    ViewState::from_json(&json)
}
