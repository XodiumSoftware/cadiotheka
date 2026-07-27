//! 3D IFC model viewer using a custom WebGL renderer.
//!
//! The viewer fetches a pre-converted GLB from the backend (`/data/projects/:id/glb`),
//! parses it with the minimal GLB parser in [`crate::utils::glb`], and renders it
//! with the custom WebGL renderer in [`crate::utils::webgl_renderer`].

use crate::utils::glb::parse_glb;
use crate::utils::webgl_renderer::{OrbitControls, Renderer};
use gloo_net::http::Request;
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
#[component]
pub fn IfcViewer(#[prop(into)] url: Signal<Option<String>>) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let (state, set_state) = signal(IfcViewerState::NoModel);

    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::new(RefCell::new(None));
    let controls: Rc<RefCell<Option<OrbitControls>>> = Rc::new(RefCell::new(None));

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
                        if let Some(renderer) = renderer.borrow().as_ref() {
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

        leptos::task::spawn_local(async move {
            match load_model_bytes(&url).await {
                Some(glb_bytes) => {
                    set_state.set(IfcViewerState::Processing);
                    let Ok(doc) = parse_glb(&glb_bytes) else {
                        set_state.set(IfcViewerState::Error);
                        return;
                    };

                    if let Some(new_renderer) = Renderer::new(canvas, &doc) {
                        let render_callback = {
                            let request_render = Rc::clone(&request_render);
                            move || request_render.borrow_mut()()
                        };
                        let new_controls = OrbitControls::attach(&new_renderer, render_callback);
                        *renderer.borrow_mut() = Some(new_renderer);
                        *controls.borrow_mut() = Some(new_controls);
                        set_state.set(IfcViewerState::Rendering);
                        request_render.borrow_mut()();
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
                IfcViewerState::Rendering => ().into_any(),
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
