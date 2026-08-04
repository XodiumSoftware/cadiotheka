//! Mouse and wheel orbit controls for the IFC viewer.
//!
//! Event handling is wired through Leptos `on:mouse:*`/`on:wheel` attributes on
//! the canvas instead of raw DOM listeners, so the framework owns listener
//! lifecycle. This module converts browser events into `three-d` control events
//! and forwards them to the renderer's pending queue.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::three_d_viewer::renderer::Renderer;
use leptos::web_sys::{MouseEvent, WheelEvent};
use std::cell::RefCell;
use std::rc::Rc;
use three_d::renderer::control::{Event, MouseButton};
use three_d_asset::PixelPoint;

/// Cross-event state for the orbit controls.
///
/// Handlers are invoked from Leptos `on:` attributes; this struct tracks the
/// pressed button and the double-click timer between events.
#[derive(Default)]
pub struct OrbitControls {
    last_button: Option<MouseButton>,
    last_press_time: f64,
}

impl OrbitControls {
    /// Handles `mousedown`, pushing a press event and resetting the view on a
    /// middle-button double click.
    ///
    /// Returns `true` when an event was recorded so the caller can request a
    /// render.
    pub fn on_mouse_down(
        &mut self,
        ev: &MouseEvent,
        renderer: &Rc<RefCell<Option<Renderer>>>,
    ) -> bool {
        ev.prevent_default();
        let Some(pending_events) = renderer_events(renderer) else {
            return false;
        };
        let position = physical_point_from_mouse(ev);
        let button = mouse_button_from_web(ev.button());
        let modifiers = modifiers_from_mouse(ev);
        let now = window_performance_now();
        let is_double_click = button == MouseButton::Middle && now - self.last_press_time <= 300.0;
        self.last_button = Some(button);
        self.last_press_time = now;
        pending_events.borrow_mut().push(Event::MousePress {
            button,
            position,
            modifiers,
            handled: false,
        });
        if is_double_click && let Some(renderer) = renderer.borrow_mut().as_mut() {
            renderer.reset_view();
        }
        true
    }

    /// Handles `mousemove`, panning with the middle button via the shift
    /// modifier.
    ///
    /// Returns `true` when an event was recorded so the caller can request a
    /// render.
    pub fn on_mouse_move(&self, ev: &MouseEvent, renderer: &Rc<RefCell<Option<Renderer>>>) -> bool {
        ev.prevent_default();
        let Some(pending_events) = renderer_events(renderer) else {
            return false;
        };
        let position = physical_point_from_mouse(ev);
        let modifiers = modifiers_from_mouse(ev);
        let delta = (ev.movement_x() as f32, ev.movement_y() as f32);
        let button = self.last_button;
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
        true
    }

    /// Handles `mouseup`, releasing the tracked button.
    ///
    /// Returns `true` when an event was recorded so the caller can request a
    /// render.
    pub fn on_mouse_up(
        &mut self,
        ev: &MouseEvent,
        renderer: &Rc<RefCell<Option<Renderer>>>,
    ) -> bool {
        let position = physical_point_from_mouse(ev);
        let button = mouse_button_from_web(ev.button());
        let modifiers = modifiers_from_mouse(ev);
        self.last_button = None;
        let Some(pending_events) = renderer_events(renderer) else {
            return false;
        };
        pending_events.borrow_mut().push(Event::MouseRelease {
            button,
            position,
            modifiers,
            handled: false,
        });
        true
    }

    /// Handles `wheel`, zooming the camera.
    ///
    /// Returns `true` when an event was recorded so the caller can request a
    /// render.
    pub fn on_wheel(ev: &WheelEvent, renderer: &Rc<RefCell<Option<Renderer>>>) -> bool {
        ev.prevent_default();
        let Some(pending_events) = renderer_events(renderer) else {
            return false;
        };
        let position = physical_point_from_wheel(ev);
        let modifiers = modifiers_from_wheel(ev);
        pending_events.borrow_mut().push(Event::MouseWheel {
            delta: (0.0, -f32::from(ev.delta_y() as i16)),
            position,
            modifiers,
            handled: false,
        });
        true
    }
}

/// Returns the renderer's pending event queue, or `None` when no renderer is
/// currently stored.
fn renderer_events(renderer: &Rc<RefCell<Option<Renderer>>>) -> Option<Rc<RefCell<Vec<Event>>>> {
    let renderer = renderer.borrow();
    renderer.as_ref().map(Renderer::pending_events)
}

fn window_performance_now() -> f64 {
    leptos::web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
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
