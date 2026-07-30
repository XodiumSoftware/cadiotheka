//! IFC model renderer built on top of the `three-d` crate.
//!
//! This module replaces the previous hand-rolled WebGL renderer. It keeps the
//! same scene framing and double-sided material behaviour, but delegates buffer
//! management, shaders, draw calls and orbit interaction to `three-d`.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::utils::glb::NodeMetadata;
#[cfg(target_arch = "wasm32")]
use crate::utils::glb::{
    build_node_metadata_map, compute_bounding_box, is_triangle_mode, material_params, read_indices,
    read_normals, read_positions, triangle_count,
};
#[cfg(target_arch = "wasm32")]
use crate::utils::math::{mat4_identity, mat4_mul};
#[cfg(target_arch = "wasm32")]
use glow;
use gltf::Gltf;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
use leptos::web_sys::HtmlCanvasElement;
use leptos::web_sys::WebGl2RenderingContext;
use leptos::web_sys::{MouseEvent, WheelEvent};
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use three_d::Gm;
use three_d::InnerSpace;
use three_d::MetricSpace;
#[cfg(target_arch = "wasm32")]
use three_d::core::render_states::{Cull, DepthTest};
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
use three_d_asset::Viewport;
#[cfg(target_arch = "wasm32")]
use three_d_asset::material::LightingModel;
#[cfg(target_arch = "wasm32")]
use three_d_asset::{Mat4, PbrMaterial, Srgba, radians, vec3};
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
    #[cfg(target_arch = "wasm32")]
    mesh_metadata: Vec<Option<NodeMetadata>>,
    total_vertices: usize,
    total_triangles: usize,
    light: DirectionalLight,
    pending_events: Rc<RefCell<Vec<Event>>>,
    theme: ViewerTheme,
    #[cfg(target_arch = "wasm32")]
    skybox: Gm<Mesh, ColorMaterial>,
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

/// Bottom and top colors for the gradient sky sphere.
#[cfg(target_arch = "wasm32")]
fn skybox_gradient(theme: ViewerTheme) -> (Srgba, Srgba) {
    match theme {
        ViewerTheme::Dark => (
            Srgba::new(245, 250, 255, 255),
            Srgba::new(66, 130, 190, 255),
        ),
        ViewerTheme::Light => (
            Srgba::new(255, 255, 255, 255),
            Srgba::new(176, 224, 230, 255),
        ),
    }
}

#[cfg(target_arch = "wasm32")]
fn lerp_srgba(a: Srgba, b: Srgba, t: f32) -> Srgba {
    let lerp = |x: u8, y: u8| -> u8 {
        let value = f32::from(x) + (f32::from(y) - f32::from(x)) * t;
        let clamped = value.clamp(0.0, 255.0);
        u8::try_from(clamped as i32).unwrap_or(0)
    };
    Srgba::new(lerp(a.r, b.r), lerp(a.g, b.g), lerp(a.b, b.b), 255)
}

/// Creates a large gradient sphere that follows the camera and acts as a skybox.
#[cfg(target_arch = "wasm32")]
fn build_skybox(context: &ThreeDContext, theme: ViewerTheme) -> Gm<Mesh, ColorMaterial> {
    let mut cpu_mesh = CpuMesh::sphere(32);
    let (bottom, top) = skybox_gradient(theme);
    let colors = match &cpu_mesh.positions {
        Positions::F32(positions) => positions
            .iter()
            .map(|p| {
                let t = (p.y + 1.0) * 0.5;
                lerp_srgba(bottom, top, t.clamp(0.0, 1.0))
            })
            .collect(),
        Positions::F64(positions) => positions
            .iter()
            .map(|p| {
                let t = ((p.y as f32) + 1.0) * 0.5;
                lerp_srgba(bottom, top, t.clamp(0.0, 1.0))
            })
            .collect(),
    };
    cpu_mesh.colors = Some(colors);

    let mesh = Mesh::new(context, &cpu_mesh);
    let cpu_material = PbrMaterial {
        name: String::new(),
        albedo: Srgba::WHITE,
        albedo_texture: None,
        metallic: 0.0,
        roughness: 1.0,
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
    let mut material = ColorMaterial::new(context, &cpu_material);
    material.render_states.write_mask.depth = false;
    material.render_states.depth_test = DepthTest::Always;
    material.render_states.cull = Cull::Back;
    Gm::new(mesh, material)
}

/// Per-mesh world-space geometry data retained for raycasting.
#[derive(Clone, Debug)]
pub struct MeshGeometry {
    /// World-space vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Optional indices describing triangles.
    pub indices: Option<Vec<u32>>,
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
            suppress_webgl_debug_renderer_info(&gl_context);
            let glow_context = glow::Context::from_webgl2_context(gl_context);
            #[allow(clippy::arc_with_non_send_sync)]
            let context = ThreeDContext::from_gl_context(Arc::new(glow_context)).ok()?;

            let scene_bounds = compute_bounding_box(gltf);
            let (camera, control) = build_framing_camera(scene_bounds.0, scene_bounds.1, canvas);

            let mut models = Vec::new();
            let mut mesh_metadata: Vec<Option<NodeMetadata>> = Vec::new();
            let mut total_vertices = 0;
            let mut total_triangles = 0;

            if let Some(scene) = gltf.default_scene() {
                let metadata_map = build_node_metadata_map(gltf);
                let identity = mat4_identity();
                for node in scene.nodes() {
                    upload_node(
                        gltf,
                        &node,
                        &identity,
                        &context,
                        &metadata_map,
                        &mut models,
                        &mut mesh_metadata,
                        &mut total_vertices,
                        &mut total_triangles,
                    );
                }
            }

            let light =
                DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.3_f32, -0.8, -0.5));
            let skybox = build_skybox(&context, ViewerTheme::default());

            Some(Self {
                context,
                camera,
                control,
                canvas: canvas.clone(),
                scene_bounds,
                models,
                mesh_metadata,
                total_vertices,
                total_triangles,
                light,
                pending_events: Rc::new(RefCell::new(Vec::new())),
                theme: ViewerTheme::default(),
                skybox,
            })
        }
    }

    /// Resets the camera and orbit target to frame the loaded model.
    #[cfg(target_arch = "wasm32")]
    pub fn reset_view(&mut self) {
        let (camera, control) =
            build_framing_camera(self.scene_bounds.0, self.scene_bounds.1, &self.canvas);
        self.camera = camera;
        self.control = control;
    }

    /// Resets the camera and orbit target to frame the loaded model.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn reset_view(&mut self) {}

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

        let eye = self.camera.position();
        let target = self.camera.target();
        let forward = eye - target;
        let right = self.camera.right_direction();
        let up = self.camera.up_orthogonal();
        let light_dir = (forward + right * 0.2 + up * 0.3).normalize();
        if light_dir.magnitude2() > 0.0 {
            self.light.direction = light_dir;
        }

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

        let target = RenderTarget::screen(&self.context, width, height);
        target.clear(ClearState::color_and_depth(
            background.0,
            background.1,
            background.2,
            background.3,
            background.4,
        ));
        #[cfg(target_arch = "wasm32")]
        {
            let eye = self.camera.position();
            let radius = self.camera.z_near() * 10.0;
            let transform = Mat4::from_translation(eye) * Mat4::from_scale(radius);
            self.skybox.set_transformation(transform);
            target.render(&self.camera, [&self.skybox], &[]);
        }
        target.render(&self.camera, objects, &[&self.light]);
    }

    /// Sets the viewer theme and re-renders on the next frame.
    #[cfg(target_arch = "wasm32")]
    pub fn set_theme(&mut self, theme: ViewerTheme) {
        self.theme = theme;
        self.skybox = build_skybox(&self.context, theme);
    }

    /// Sets the viewer theme and re-renders on the next frame.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_theme(&mut self, theme: ViewerTheme) {
        self.theme = theme;
    }

    /// Returns the current viewer theme.
    pub fn theme(&self) -> ViewerTheme {
        self.theme
    }

    /// Returns the metadata attached to the closest mesh under the given
    /// screen-space pixel coordinate, or `None` if nothing was hit.
    #[cfg(target_arch = "wasm32")]
    pub fn pick(
        &self,
        pixel: three_d_asset::PixelPoint,
        geometries: &[MeshGeometry],
    ) -> Option<PickResult> {
        let (width, height) = canvas_size(&self.canvas);
        if width == 0 || height == 0 || geometries.len() != self.mesh_metadata.len() {
            return None;
        }
        let ray_origin = self.camera.position_at_pixel(pixel);
        let ray_dir = self.camera.view_direction_at_pixel(pixel).normalize();

        let mut best: Option<PickResult> = None;
        for (idx, geometry) in geometries.iter().enumerate() {
            let Some(metadata) = self.mesh_metadata.get(idx).cloned().flatten() else {
                continue;
            };
            let Some(distance) = ray_intersect_triangles(
                ray_origin,
                ray_dir,
                &geometry.positions,
                geometry.indices.as_deref(),
            ) else {
                continue;
            };
            let pick = PickResult {
                metadata: Some(metadata),
                distance,
            };
            best = Some(best.map_or(pick.clone(), |b| {
                if b.distance < pick.distance { b } else { pick }
            }));
        }
        best
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn pick(
        &self,
        _pixel: three_d_asset::PixelPoint,
        _geometries: &[MeshGeometry],
    ) -> Option<PickResult> {
        None
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

/// Patches `getExtension` on a WebGL2 context so that Firefox does not emit a
/// deprecation warning for `WEBGL_debug_renderer_info`.
///
/// `glow` 0.17 eagerly queries this extension during context creation, but it
/// is deprecated in Firefox. Returning `null` for it is safe because neither
/// `glow` nor `three-d` consume the extension object.
#[cfg(target_arch = "wasm32")]
fn suppress_webgl_debug_renderer_info(context: &WebGl2RenderingContext) {
    let Ok(original) = Reflect::get(context, &"getExtension".into())
        .and_then(wasm_bindgen::JsCast::dyn_into::<Function>)
    else {
        return;
    };
    let Ok(true) = Reflect::set(context, &"getExtensionOriginal".into(), &original) else {
        return;
    };
    let wrapper = Function::new_with_args(
        "name",
        r#"
            if (name === "WEBGL_debug_renderer_info") {
                return null;
            }
            return this.getExtensionOriginal(name);
        "#,
    );
    let _ = Reflect::set(context, &"getExtension".into(), &wrapper);
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

    // Fit the scene's largest extent into the viewport using the smaller of the
    // horizontal and vertical fields of view, with a small padding margin so the
    // model does not touch the edges of the canvas.
    let fov_y = std::f32::consts::PI * 0.25;
    let (width, height) = canvas_size(canvas);
    let aspect = if height == 0 {
        1.0
    } else {
        width as f32 / height as f32
    };
    let half_fov_y = fov_y * 0.5;
    let fov_x = 2.0 * (aspect * half_fov_y.tan()).atan();
    let limiting_fov = fov_y.min(fov_x);
    let padding = 1.2;
    let distance = (max_size * 0.5 / (limiting_fov * 0.5).tan()) * padding;

    let yaw = std::f32::consts::PI * 0.25;
    let pitch = std::f32::consts::PI * 0.15;
    let cos_pitch = pitch.cos();
    let eye = [
        center[0] + distance * cos_pitch * yaw.sin(),
        center[1] + distance * pitch.sin(),
        center[2] + distance * cos_pitch * yaw.cos(),
    ];
    let near = max_size * 0.001;
    let far = max_size * 1_000.0;

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
#[allow(clippy::too_many_arguments)]
fn upload_node(
    gltf: &Gltf,
    node: &gltf::Node<'_>,
    parent_transform: &[[f32; 4]; 4],
    context: &ThreeDContext,
    metadata_map: &std::collections::HashMap<usize, NodeMetadata>,
    models: &mut Vec<Box<dyn Object>>,
    mesh_metadata: &mut Vec<Option<NodeMetadata>>,
    total_vertices: &mut usize,
    total_triangles: &mut usize,
) {
    let transform = mat4_mul(parent_transform, &node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        let metadata = metadata_map.get(&node.index()).cloned();
        for primitive in mesh.primitives() {
            upload_primitive(
                gltf,
                &primitive,
                &transform,
                context,
                metadata.clone(),
                models,
                mesh_metadata,
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
            metadata_map,
            models,
            mesh_metadata,
            total_vertices,
            total_triangles,
        );
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
fn upload_primitive(
    gltf: &Gltf,
    primitive: &gltf::Primitive<'_>,
    transform: &[[f32; 4]; 4],
    context: &ThreeDContext,
    metadata: Option<NodeMetadata>,
    models: &mut Vec<Box<dyn Object>>,
    mesh_metadata: &mut Vec<Option<NodeMetadata>>,
    total_vertices: &mut usize,
    total_triangles: &mut usize,
) {
    let mode = primitive.mode();
    if !is_triangle_mode(mode) {
        mesh_metadata.push(metadata);
        return;
    }

    let Some(positions) = read_positions(gltf, primitive, transform) else {
        mesh_metadata.push(metadata);
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
    mesh_metadata.push(metadata);

    let index_count = indices.as_ref().map_or(0, std::vec::Vec::len);
    *total_vertices += position_count;
    *total_triangles += triangle_count(mode, index_count, position_count);
}

/// Metadata returned when picking an object in the viewer.
#[derive(Clone, Debug)]
pub struct PickResult {
    /// Metadata of the closest picked mesh, if available.
    pub metadata: Option<NodeMetadata>,
    /// World-space distance from the camera to the hit point.
    pub distance: f32,
}

/// Picks the closest mesh under a screen-space point by casting a ray through
/// the camera frustum.
#[cfg(target_arch = "wasm32")]
fn ray_intersect_triangles(
    ray_origin: three_d_asset::Vec3,
    ray_dir: three_d_asset::Vec3,
    positions: &[[f32; 3]],
    indices: Option<&[u32]>,
) -> Option<f32> {
    let triangles: Vec<[u32; 3]> = match indices {
        Some(idx) => idx.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
        None => (0..positions.len() as u32)
            .step_by(3)
            .map(|i| [i, i + 1, i + 2])
            .collect(),
    };

    let mut closest: Option<f32> = None;
    for &[ia, ib, ic] in &triangles {
        let a = three_d_asset::vec3(
            positions[ia as usize][0],
            positions[ia as usize][1],
            positions[ia as usize][2],
        );
        let b = three_d_asset::vec3(
            positions[ib as usize][0],
            positions[ib as usize][1],
            positions[ib as usize][2],
        );
        let c = three_d_asset::vec3(
            positions[ic as usize][0],
            positions[ic as usize][1],
            positions[ic as usize][2],
        );
        if let Some(t) = triangle_intersection(ray_origin, ray_dir, a, b, c) {
            closest = Some(closest.map_or(t, |c| c.min(t)));
        }
    }
    closest
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::many_single_char_names)]
fn triangle_intersection(
    ray_origin: three_d_asset::Vec3,
    ray_dir: three_d_asset::Vec3,
    v0: three_d_asset::Vec3,
    v1: three_d_asset::Vec3,
    v2: three_d_asset::Vec3,
) -> Option<f32> {
    let epsilon = 1e-6;
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray_dir.cross(edge2);
    let det = edge1.dot(h);
    if det.abs() < epsilon {
        return None;
    }
    let inv_det = 1.0 / det;
    let s = ray_origin - v0;
    let u = inv_det * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = inv_det * ray_dir.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv_det * edge2.dot(q);
    if t > epsilon { Some(t) } else { None }
}
