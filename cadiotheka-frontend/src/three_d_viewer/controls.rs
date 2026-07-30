//! Mouse and wheel orbit controls for the IFC viewer.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::three_d_viewer::renderer::Renderer;
use leptos::web_sys::{HtmlCanvasElement, MouseEvent, WheelEvent};
use std::cell::RefCell;
use std::rc::Rc;
use three_d::renderer::control::{Event, MouseButton};
use three_d_asset::PixelPoint;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;

/// Mouse interaction handle that keeps event closures alive.
pub struct OrbitControls {
    _canvas: HtmlCanvasElement,
    _closures: Vec<JsValue>,
    /// Tracks the currently pressed mouse button so motion events carry the
    /// correct button for `three-d::OrbitControl`.
    #[allow(dead_code)]
    last_button: Rc<RefCell<Option<MouseButton>>>,
    /// Last time a mouse button was pressed, used to detect double clicks.
    #[allow(dead_code)]
    last_press_time: Rc<RefCell<f64>>,
}

impl OrbitControls {
    /// Attaches orbit mouse listeners to the canvas.
    ///
    /// # Panics
    ///
    /// Panics if `renderer` does not currently contain a `Renderer`. The caller
    /// must store the renderer before calling this method.
    pub fn attach<F: FnMut() + 'static>(
        renderer: &Rc<RefCell<Option<Renderer>>>,
        request_render: F,
    ) -> Self {
        let renderer_guard = renderer.borrow();
        let Some(renderer_ref) = renderer_guard.as_ref() else {
            panic!("OrbitControls::attach called without a stored renderer");
        };
        let canvas = renderer_ref.canvas.clone();
        let pending_events = renderer_ref.pending_events();
        let request_render = Rc::new(RefCell::new(request_render));
        let last_button: Rc<RefCell<Option<MouseButton>>> = Rc::new(RefCell::new(None));
        let last_press_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.0));

        let on_mouse_down = {
            let pending_events = Rc::clone(&pending_events);
            let request_render = Rc::clone(&request_render);
            let last_button = Rc::clone(&last_button);
            let last_press_time = Rc::clone(&last_press_time);
            let renderer_clone = Rc::clone(renderer);
            Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
                let position = physical_point_from_mouse(&ev);
                let button = mouse_button_from_web(ev.button());
                let modifiers = modifiers_from_mouse(&ev);
                let now = leptos::web_sys::window()
                    .and_then(|w| w.performance())
                    .map_or(0.0, |p| p.now());
                let is_double_click =
                    button == MouseButton::Middle && now - *last_press_time.borrow() <= 300.0;
                *last_button.borrow_mut() = Some(button);
                *last_press_time.borrow_mut() = now;
                pending_events.borrow_mut().push(Event::MousePress {
                    button,
                    position,
                    modifiers,
                    handled: false,
                });
                if is_double_click {
                    let mut renderer_ref = renderer_clone.borrow_mut();
                    if let Some(renderer) = renderer_ref.as_mut() {
                        renderer.reset_view();
                    }
                }
                request_render.borrow_mut()();
            })
            .into_js_value()
        };

        let on_mouse_move = {
            let pending_events = Rc::clone(&pending_events);
            let request_render = Rc::clone(&request_render);
            let last_button = Rc::clone(&last_button);
            Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
                let position = physical_point_from_mouse(&ev);
                let modifiers = modifiers_from_mouse(&ev);
                let delta = (ev.movement_x() as f32, ev.movement_y() as f32);
                let button = *last_button.borrow();
                // Holding the middle mouse button behaves like shift: pan instead of orbit.
                let modifiers = if button == Some(MouseButton::Middle) {
                    three_d::renderer::control::Modifiers {
                        shift: true,
                        ..modifiers
                    }
                } else {
                    modifiers
                };
                pending_events.borrow_mut().push(Event::MouseMotion {
                    button,
                    delta,
                    position,
                    modifiers,
                    handled: false,
                });
                request_render.borrow_mut()();
            })
            .into_js_value()
        };

        let on_mouse_up = {
            let pending_events = Rc::clone(&pending_events);
            let request_render = Rc::clone(&request_render);
            let last_button = Rc::clone(&last_button);
            Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
                let position = physical_point_from_mouse(&ev);
                let button = mouse_button_from_web(ev.button());
                let modifiers = modifiers_from_mouse(&ev);
                *last_button.borrow_mut() = None;
                pending_events.borrow_mut().push(Event::MouseRelease {
                    button,
                    position,
                    modifiers,
                    handled: false,
                });
                request_render.borrow_mut()();
            })
            .into_js_value()
        };

        let on_wheel = {
            let pending_events = Rc::clone(&pending_events);
            let request_render = Rc::clone(&request_render);
            Closure::<dyn FnMut(WheelEvent)>::new(move |ev: WheelEvent| {
                ev.prevent_default();
                let position = physical_point_from_wheel(&ev);
                let modifiers = modifiers_from_wheel(&ev);
                pending_events.borrow_mut().push(Event::MouseWheel {
                    delta: (0.0, -f32::from(ev.delta_y() as i16)),
                    position,
                    modifiers,
                    handled: false,
                });
                request_render.borrow_mut()();
            })
            .into_js_value()
        };

        canvas
            .add_event_listener_with_callback("mousedown", on_mouse_down.unchecked_ref())
            .ok();
        canvas
            .add_event_listener_with_callback("mousemove", on_mouse_move.unchecked_ref())
            .ok();
        canvas
            .add_event_listener_with_callback("mouseup", on_mouse_up.unchecked_ref())
            .ok();
        canvas
            .add_event_listener_with_callback("wheel", on_wheel.unchecked_ref())
            .ok();

        Self {
            _canvas: canvas,
            _closures: vec![on_mouse_down, on_mouse_move, on_mouse_up, on_wheel],
            last_button,
            last_press_time,
        }
    }
}

fn mouse_button_from_web(button: i16) -> MouseButton {
    match button {
        2 => MouseButton::Right,
        1 => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

fn physical_point_from_mouse(ev: &MouseEvent) -> PixelPoint {
    PixelPoint {
        x: ev.client_x() as f32,
        y: ev.client_y() as f32,
    }
}

fn physical_point_from_wheel(ev: &WheelEvent) -> PixelPoint {
    PixelPoint {
        x: ev.client_x() as f32,
        y: ev.client_y() as f32,
    }
}

fn modifiers_from_mouse(ev: &MouseEvent) -> three_d::renderer::control::Modifiers {
    three_d::renderer::control::Modifiers {
        shift: ev.shift_key(),
        ctrl: ev.ctrl_key(),
        alt: ev.alt_key(),
        command: ev.meta_key(),
    }
}

fn modifiers_from_wheel(ev: &WheelEvent) -> three_d::renderer::control::Modifiers {
    three_d::renderer::control::Modifiers {
        shift: ev.shift_key(),
        ctrl: ev.ctrl_key(),
        alt: ev.alt_key(),
        command: ev.meta_key(),
    }
}
