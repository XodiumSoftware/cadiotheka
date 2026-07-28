//! 3D IFC model viewer using the `three-d` renderer.
//!
//! The viewer fetches a pre-converted GLB from the backend (`/data/projects/:id/glb`),
//! parses it with the `gltf` crate via [`crate::utils::glb`], and renders it
//! with the `three-d` renderer in [`crate::utils::three_d_renderer`].

use crate::components::ui::toolbar_button::ToolbarButton;
use crate::utils::glb::Gltf;
use crate::utils::three_d_renderer::{OrbitControls, Renderer, ViewerTheme};
use gloo_net::http::Request;
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
#[component]
pub fn IfcViewer(#[prop(into)] url: Signal<Option<String>>) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let (state, set_state) = signal(IfcViewerState::NoModel);

    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::new(RefCell::new(None));
    let controls: Rc<RefCell<Option<OrbitControls>>> = Rc::new(RefCell::new(None));

    let (show_debug, set_show_debug) = signal(false);
    let (debug_text, set_debug_text) = signal(String::new());
    let (fps, set_fps) = signal(0.0_f64);
    let (theme, set_theme) = signal(ViewerTheme::Dark);
    let dirty = Rc::new(RefCell::new(false));
    let pending_frame = Rc::new(RefCell::new(false));
    let animation_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    let request_render: Rc<RefCell<dyn FnMut()>> = {
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
                Closure::<dyn FnMut()>::new(move || {
                    *pending_frame.borrow_mut() = false;
                    *animation_handle.borrow_mut() = None;
                    if *dirty.borrow() {
                        if let Some(renderer) = renderer.borrow_mut().as_mut() {
                            renderer.render();
                        }
                        *dirty.borrow_mut() = false;
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
            set_debug_text.set(text);

            let window = leptos::web_sys::window().and_then(|w| w.performance());
            if let Some(performance) = window {
                let now: f64 = performance.now();
                *frame_count.borrow_mut() += 1;
                if now - *last_fps_update.borrow() >= 500.0 {
                    let fps_value = f64::from(*frame_count.borrow()) * 1000.0
                        / (now - *last_fps_update.borrow());
                    set_fps.set(fps_value);
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
        move |_| {
            let theme_value = theme.get();
            if let Some(renderer) = renderer.borrow_mut().as_mut() {
                renderer.set_theme(theme_value);
                request_render.borrow_mut()();
            }
        }
    });

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        let Some(url) = url.get() else {
            set_state.set(IfcViewerState::NoModel);
            return;
        };

        set_state.set(IfcViewerState::Loading);
        let renderer = Rc::clone(&renderer);
        let controls = Rc::clone(&controls);
        let set_state = set_state;
        let request_render = Rc::clone(&request_render);
        let update_debug = update_debug.clone();

        leptos::task::spawn_local(async move {
            match load_model_bytes(&url).await {
                Some(glb_bytes) => {
                    set_state.set(IfcViewerState::Processing);
                    let Ok(gltf) = Gltf::from_slice(&glb_bytes) else {
                        set_state.set(IfcViewerState::Error);
                        return;
                    };

                    if let Some(new_renderer) = Renderer::new(&canvas, &gltf) {
                        let render_callback = {
                            let request_render = Rc::clone(&request_render);
                            let update_debug = update_debug.clone();
                            move || {
                                request_render.borrow_mut()();
                                update_debug();
                            }
                        };
                        let new_controls = OrbitControls::attach(&new_renderer, render_callback);
                        *renderer.borrow_mut() = Some(new_renderer);
                        *controls.borrow_mut() = Some(new_controls);
                        set_state.set(IfcViewerState::Rendering);
                        request_render.borrow_mut()();
                        update_debug();
                    } else {
                        set_state.set(IfcViewerState::Error);
                    }
                }
                None => {
                    set_state.set(IfcViewerState::Error);
                }
            }
        });
    });

    view! {
        <div class="relative w-full h-full min-h-[20rem] rounded-none border border-base-content/10 bg-base-200/20 overflow-hidden">
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
                    <div class="absolute top-2 left-2 right-2 z-10 flex items-center justify-between gap-2 pointer-events-none">
                        <div class="bg-base-100/80 backdrop-blur text-xs font-mono p-2 rounded border border-base-content/10 text-base-content/70 pointer-events-auto flex gap-2 items-center">
                            {move || format!("{fps:.1} FPS", fps = fps.get())}
                        </div>
                        <div class="bg-base-100/80 backdrop-blur rounded border border-base-content/10 pointer-events-auto flex gap-1 items-center p-1">
                            <ToolbarButton
                                label="Toggle theme"
                                on_click=Callback::new(move |()| {
                                    set_theme.update(|t| {
                                        *t = match *t {
                                            ViewerTheme::Dark => ViewerTheme::Light,
                                            ViewerTheme::Light => ViewerTheme::Dark,
                                        };
                                    });
                                })
                            >
                                {move || match theme.get() {
                                    ViewerTheme::Dark => "☀",
                                    ViewerTheme::Light => "🌙",
                                }}
                            </ToolbarButton>
                            <ToolbarButton
                                label="Toggle debug overlay"
                                on_click=Callback::new(move |()| {
                                    set_show_debug.update(|v| *v = !*v);
                                })
                            >
                                {move || if show_debug.get() { "🐞" } else { "🔍" }}
                            </ToolbarButton>
                        </div>
                    </div>
                    {move || show_debug.get().then(|| view! {
                        <div class="absolute top-12 left-2 z-10 max-w-[20rem] bg-base-100/90 backdrop-blur text-xs font-mono p-3 rounded border border-base-content/10 text-base-content/80 whitespace-pre-wrap">
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
