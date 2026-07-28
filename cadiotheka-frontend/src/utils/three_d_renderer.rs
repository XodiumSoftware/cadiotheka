//! IFC model renderer built on top of the `three-d` crate.
//!
//! This module replaces the previous hand-rolled WebGL renderer. It keeps the
//! same scene framing and double-sided material behaviour, but delegates buffer
//! management, shaders, draw calls and orbit interaction to `three-d`.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

#[cfg(target_arch = "wasm32")]
use crate::utils::glb::{
    compute_bounding_box, is_triangle_mode, material_params, read_indices, read_normals,
    read_positions, triangle_count,
};
#[cfg(target_arch = "wasm32")]
use crate::utils::math::{mat4_identity, mat4_mul};
#[cfg(target_arch = "wasm32")]
use glow;
use gltf::Gltf;
use leptos::web_sys::HtmlCanvasElement;
use leptos::web_sys::WebGl2RenderingContext;
use leptos::web_sys::{MouseEvent, WheelEvent};
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use three_d::Gm;
use three_d::MetricSpace;
#[cfg(target_arch = "wasm32")]
use three_d::core::render_states::Cull;
use three_d::core::{ClearState, Context as ThreeDContext, RenderTarget};
#[cfg(target_arch = "wasm32")]
use three_d::renderer::Mesh;
#[cfg(target_arch = "wasm32")]
use three_d::renderer::PhysicalMaterial;
use three_d::renderer::control::{Event, MouseButton, OrbitControl};
#[cfg(target_arch = "wasm32")]
use three_d::renderer::geometry::{CpuMesh, Indices, Positions};
#[cfg(target_arch = "wasm32")]
use three_d::renderer::material::ColorMaterial;
use three_d::renderer::{Camera as ThreeDCamera, DirectionalLight, Object};
use three_d_asset::Srgba as SharedSrgba;
use three_d_asset::Viewport;
#[cfg(target_arch = "wasm32")]
use three_d_asset::material::LightingModel;
#[cfg(target_arch = "wasm32")]
use three_d_asset::{PbrMaterial, Srgba, radians, vec3};
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
    /// `request_render` is called whenever an input event is queued.
    pub fn attach<F: FnMut() + 'static>(renderer: &Renderer, request_render: F) -> Self {
        let canvas = renderer.canvas.clone();
        let pending_events = renderer.pending_events();
        let request_render = Rc::new(RefCell::new(request_render));
        let last_button: Rc<RefCell<Option<MouseButton>>> = Rc::new(RefCell::new(None));
        let last_press_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.0));

        let on_mouse_down = {
            let pending_events = Rc::clone(&pending_events);
            let request_render = Rc::clone(&request_render);
            let last_button = Rc::clone(&last_button);
            let last_press_time = Rc::clone(&last_press_time);
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
                    pending_events.borrow_mut().push(Event::MouseWheel {
                        delta: (0.0, -10.0),
                        position,
                        modifiers,
                        handled: false,
                    });
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

fn physical_point_from_mouse(ev: &MouseEvent) -> three_d_asset::PixelPoint {
    three_d_asset::PixelPoint {
        x: ev.client_x() as f32,
        y: ev.client_y() as f32,
    }
}

fn physical_point_from_wheel(ev: &WheelEvent) -> three_d_asset::PixelPoint {
    three_d_asset::PixelPoint {
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

/// `three-d` renderer for a parsed GLB document.
pub struct Renderer {
    context: ThreeDContext,
    camera: ThreeDCamera,
    control: OrbitControl,
    canvas: HtmlCanvasElement,
    scene_bounds: ([f32; 3], [f32; 3]),
    models: Vec<Box<dyn Object>>,
    total_vertices: usize,
    total_triangles: usize,
    light: DirectionalLight,
    pending_events: Rc<RefCell<Vec<Event>>>,
    theme: ViewerTheme,
}

/// Rendering theme for the viewer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewerTheme {
    /// Dark background, bright light.
    #[default]
    Dark,
    /// Light background, darker light.
    Light,
}

impl Renderer {
    /// Creates a renderer for the given canvas and glTF document.
    pub fn new(canvas: &HtmlCanvasElement, gltf: &Gltf) -> Option<Self> {
        let gl_context = canvas
            .get_context("webgl2")
            .ok()??
            .dyn_into::<WebGl2RenderingContext>()
            .ok()?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = gl_context;
            let _ = gltf;
            None
        }

        #[cfg(target_arch = "wasm32")]
        {
            let glow_context = glow::Context::from_webgl2_context(gl_context);
            #[allow(clippy::arc_with_non_send_sync)]
            let context = ThreeDContext::from_gl_context(Arc::new(glow_context)).ok()?;

            let scene_bounds = compute_bounding_box(gltf);
            let (camera, control) = build_framing_camera(scene_bounds.0, scene_bounds.1, canvas);

            let mut models = Vec::new();
            let mut total_vertices = 0;
            let mut total_triangles = 0;

            if let Some(scene) = gltf.default_scene() {
                let identity = mat4_identity();
                for node in scene.nodes() {
                    upload_node(
                        gltf,
                        &node,
                        &identity,
                        &context,
                        &mut models,
                        &mut total_vertices,
                        &mut total_triangles,
                    );
                }
            }

            let light =
                DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.3_f32, -0.8, -0.5));

            Some(Self {
                context,
                camera,
                control,
                canvas: canvas.clone(),
                scene_bounds,
                models,
                total_vertices,
                total_triangles,
                light,
                pending_events: Rc::new(RefCell::new(Vec::new())),
                theme: ViewerTheme::default(),
            })
        }
    }

    /// Renders the scene once.
    pub fn render(&mut self) {
        let (width, height) = canvas_size(&self.canvas);
        if width == 0 || height == 0 {
            return;
        }
        self.resize();

        let mut events = self
            .pending_events
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();
        self.handle_pan_events(&mut events);
        self.control.handle_events(&mut self.camera, &mut events);

        let viewport = Viewport::new_at_origo(width, height);
        self.camera.set_viewport(viewport);

        let objects: Vec<&dyn Object> = self
            .models
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();
        let (background, light_intensity) = match self.theme {
            ViewerTheme::Dark => ((0.05, 0.05, 0.05, 1.0, 1.0), 1.0),
            ViewerTheme::Light => ((0.95, 0.95, 0.95, 1.0, 1.0), 1.3),
        };
        self.light.intensity = light_intensity;
        RenderTarget::screen(&self.context, width, height)
            .clear(ClearState::color_and_depth(
                background.0,
                background.1,
                background.2,
                background.3,
                background.4,
            ))
            .render(&self.camera, objects, &[&self.light]);
    }

    /// Sets the viewer theme and re-renders on the next frame.
    pub fn set_theme(&mut self, theme: ViewerTheme) {
        if self.theme != theme {
            self.theme = theme;
            self.light.color = match theme {
                ViewerTheme::Dark => SharedSrgba::WHITE,
                ViewerTheme::Light => SharedSrgba::BLACK,
            };
        }
    }

    /// Returns the current viewer theme.
    pub fn theme(&self) -> ViewerTheme {
        self.theme
    }

    /// Handles shift-drag panning before delegating rotation/zoom to `OrbitControl`.
    fn handle_pan_events(&mut self, events: &mut [Event]) {
        let viewport_height = self.canvas.client_height().max(1) as f32;
        let distance = self.camera.position().distance(self.camera.target());
        let scale = distance * (Self::FOV_Y * 0.5).tan() * 2.0 / viewport_height;

        for event in events.iter_mut() {
            let Event::MouseMotion {
                delta,
                modifiers,
                handled,
                ..
            } = event
            else {
                continue;
            };
            if *handled || !modifiers.shift {
                continue;
            }
            *handled = true;

            let right = self.camera.right_direction();
            let up = self.camera.up_orthogonal();
            let shift = right * (-delta.0 * scale) + up * (delta.1 * scale);
            self.camera.translate(shift);
            self.control.target += shift;
        }
    }

    /// Vertical field of view in radians.
    const FOV_Y: f32 = std::f32::consts::PI * 0.25;

    /// Returns a reference to the `three-d` camera.
    pub fn camera(&self) -> &ThreeDCamera {
        &self.camera
    }

    /// Returns a clone of the shared pending-events queue.
    fn pending_events(&self) -> Rc<RefCell<Vec<Event>>> {
        Rc::clone(&self.pending_events)
    }

    /// Returns the axis-aligned world-space bounds computed when the document was loaded.
    pub fn scene_bounds(&self) -> (&[f32; 3], &[f32; 3]) {
        (&self.scene_bounds.0, &self.scene_bounds.1)
    }

    /// Returns the number of uploaded primitives.
    pub fn primitive_count(&self) -> usize {
        self.models.len()
    }

    /// Returns the total number of uploaded vertices across all primitives.
    pub fn total_vertices(&self) -> usize {
        self.total_vertices
    }

    /// Returns the total number of triangles across all primitives.
    pub fn total_triangles(&self) -> usize {
        self.total_triangles
    }

    /// Resizes the canvas backing store to match its display size.
    pub fn resize(&self) {
        let (width, height) = canvas_size(&self.canvas);
        if self.canvas.width() != width || self.canvas.height() != height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
        }
    }
}

fn canvas_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let width = u32::try_from(canvas.client_width()).unwrap_or(0);
    let height = u32::try_from(canvas.client_height()).unwrap_or(0);
    (width.max(1), height.max(1))
}

#[cfg(target_arch = "wasm32")]
fn build_framing_camera(
    min: [f32; 3],
    max: [f32; 3],
    canvas: &HtmlCanvasElement,
) -> (ThreeDCamera, OrbitControl) {
    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let size = [
        (max[0] - min[0]).abs(),
        (max[1] - min[1]).abs(),
        (max[2] - min[2]).abs(),
    ];
    let max_size = size[0].max(size[1]).max(size[2]).max(1.0);
    let distance = max_size * 2.5;
    let yaw = std::f32::consts::PI * 0.25;
    let pitch = std::f32::consts::PI * 0.15;
    let cos_pitch = pitch.cos();
    let eye = [
        center[0] + distance * cos_pitch * yaw.sin(),
        center[1] + distance * pitch.sin(),
        center[2] + distance * cos_pitch * yaw.cos(),
    ];
    let fov_y = std::f32::consts::PI * 0.25;
    let near = max_size * 0.001;
    let far = max_size * 1_000.0;

    let (width, height) = canvas_size(canvas);
    let viewport = Viewport::new_at_origo(width, height);

    let camera = ThreeDCamera::new_perspective(
        viewport,
        vec3(eye[0], eye[1], eye[2]),
        vec3(center[0], center[1], center[2]),
        vec3(0.0_f32, 1.0, 0.0),
        radians(fov_y),
        near,
        far,
    );
    let control = OrbitControl::new(
        vec3(center[0], center[1], center[2]),
        max_size * 0.001,
        max_size * 1_000.0,
    );
    (camera, control)
}

#[cfg(target_arch = "wasm32")]
fn upload_node(
    gltf: &Gltf,
    node: &gltf::Node<'_>,
    parent_transform: &[[f32; 4]; 4],
    context: &ThreeDContext,
    models: &mut Vec<Box<dyn Object>>,
    total_vertices: &mut usize,
    total_triangles: &mut usize,
) {
    let transform = mat4_mul(parent_transform, &node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            upload_primitive(
                gltf,
                &primitive,
                &transform,
                context,
                models,
                total_vertices,
                total_triangles,
            );
        }
    }

    for child in node.children() {
        upload_node(
            gltf,
            &child,
            &transform,
            context,
            models,
            total_vertices,
            total_triangles,
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn upload_primitive(
    gltf: &Gltf,
    primitive: &gltf::Primitive<'_>,
    transform: &[[f32; 4]; 4],
    context: &ThreeDContext,
    models: &mut Vec<Box<dyn Object>>,
    total_vertices: &mut usize,
    total_triangles: &mut usize,
) {
    let mode = primitive.mode();
    if !is_triangle_mode(mode) {
        return;
    }

    let Some(positions) = read_positions(gltf, primitive, transform) else {
        return;
    };
    let normals = read_normals(gltf, primitive, transform);
    let position_count = positions.len();
    let indices = read_indices(gltf, primitive);

    let cpu_mesh = CpuMesh {
        positions: Positions::F32(positions.iter().map(|p| vec3(p[0], p[1], p[2])).collect()),
        indices: indices
            .as_ref()
            .map_or(Indices::None, |i| Indices::U32(i.clone())),
        normals: Some(normals.iter().map(|n| vec3(n[0], n[1], n[2])).collect()),
        tangents: None,
        uvs: None,
        colors: None,
    };

    let mesh = Mesh::new(context, &cpu_mesh);

    let material_info = material_params(&primitive.material());
    let cpu_material = PbrMaterial {
        name: String::new(),
        albedo: Srgba::from(material_info.base_color_factor),
        albedo_texture: None,
        metallic: material_info.metallic_factor,
        roughness: material_info.roughness_factor,
        occlusion_metallic_roughness_texture: None,
        metallic_roughness_texture: None,
        occlusion_strength: 1.0,
        occlusion_texture: None,
        normal_scale: 1.0,
        normal_texture: None,
        emissive: Srgba::BLACK,
        emissive_texture: None,
        alpha_cutout: None,
        lighting_model: LightingModel::Blinn,
        index_of_refraction: 1.5,
        transmission: 0.0,
        transmission_texture: None,
    };

    let model: Box<dyn Object> = if material_info.unlit {
        let mut material = ColorMaterial::new(context, &cpu_material);
        material.render_states.cull = Cull::None;
        Box::new(Gm::new(mesh, material))
    } else {
        let mut material = PhysicalMaterial::new(context, &cpu_material);
        material.render_states.cull = Cull::None;
        Box::new(Gm::new(mesh, material))
    };

    models.push(model);

    let index_count = indices.as_ref().map_or(0, std::vec::Vec::len);
    *total_vertices += position_count;
    *total_triangles += triangle_count(mode, index_count, position_count);
}
