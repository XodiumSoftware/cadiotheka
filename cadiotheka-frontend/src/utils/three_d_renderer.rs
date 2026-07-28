//! IFC model renderer built on top of the `three-d` crate.
//!
//! This module replaces the previous hand-rolled WebGL renderer. It keeps the
//! same orbit camera and mouse interaction behaviour, but delegates buffer
//! management, shaders and draw calls to `three-d`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

#[cfg(target_arch = "wasm32")]
use crate::utils::glb::{
    compute_bounding_box, is_triangle_mode, material_params, read_indices, read_normals,
    read_positions, triangle_count,
};
use crate::utils::math::{cross_vec3, normalize_vec3};
#[cfg(target_arch = "wasm32")]
use crate::utils::math::{mat4_identity, mat4_mul};
#[cfg(target_arch = "wasm32")]
use glow;
use gltf::Gltf;
use leptos::web_sys::{HtmlCanvasElement, MouseEvent, WebGl2RenderingContext, WheelEvent};
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use three_d::Gm;
#[cfg(target_arch = "wasm32")]
use three_d::core::render_states::Cull;
use three_d::core::{ClearState, Context as ThreeDContext, RenderTarget};
#[cfg(target_arch = "wasm32")]
use three_d::renderer::Mesh;
#[cfg(target_arch = "wasm32")]
use three_d::renderer::PhysicalMaterial;
#[cfg(target_arch = "wasm32")]
use three_d::renderer::geometry::{CpuMesh, Indices, Positions};
#[cfg(target_arch = "wasm32")]
use three_d::renderer::material::ColorMaterial;
use three_d::renderer::{Camera as ThreeDCamera, DirectionalLight, Object};
#[cfg(target_arch = "wasm32")]
use three_d_asset::material::LightingModel;
#[cfg(target_arch = "wasm32")]
use three_d_asset::{PbrMaterial, Srgba};
use three_d_asset::{Viewport, radians, vec3};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;

/// Orbit camera state.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Point the camera orbits around.
    pub target: [f32; 3],
    /// Distance from target.
    pub distance: f32,
    /// Horizontal rotation in radians.
    pub yaw: f32,
    /// Vertical rotation in radians, clamped to avoid flipping.
    pub pitch: f32,
    /// Field of view in radians.
    pub fov_y: f32,
    /// Near plane.
    pub near: f32,
    /// Far plane.
    pub far: f32,
}

impl Camera {
    /// Creates a camera framing the given bounding box.
    pub fn framing_bounding_box(min: [f32; 3], max: [f32; 3]) -> Self {
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

        Self {
            target: center,
            distance,
            yaw: std::f32::consts::PI * 0.25,
            pitch: std::f32::consts::PI * 0.15,
            fov_y: std::f32::consts::PI * 0.25,
            near: max_size * 0.001,
            far: max_size * 1_000.0,
        }
    }

    /// Eye position in world space.
    pub fn eye(&self) -> [f32; 3] {
        let cos_pitch = self.pitch.cos();
        let dx = self.distance * cos_pitch * self.yaw.sin();
        let dy = self.distance * self.pitch.sin();
        let dz = self.distance * cos_pitch * self.yaw.cos();
        [
            self.target[0] + dx,
            self.target[1] + dy,
            self.target[2] + dz,
        ]
    }

    /// Rotates around the target from a drag delta.
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.005;
        self.yaw += delta_x * sensitivity;
        self.pitch += delta_y * sensitivity;
        self.pitch = self
            .pitch
            .clamp(-std::f32::consts::PI * 0.49, std::f32::consts::PI * 0.49);
    }

    /// Zooms by changing the distance.
    ///
    /// Positive `delta` moves the camera farther away (zoom out), negative moves
    /// it closer (zoom in).
    pub fn zoom(&mut self, delta: f32) {
        let factor = 1.0 + delta * 0.001;
        let new_distance = self.distance * factor.clamp(0.8, 1.25);
        self.distance = new_distance.clamp(self.distance * 0.01, self.distance * 10.0);
    }

    /// Pans the target in the camera plane.
    pub fn pan(&mut self, delta_x: f32, delta_y: f32, viewport_height: f32) {
        let eye = self.eye();
        let forward = [
            self.target[0] - eye[0],
            self.target[1] - eye[1],
            self.target[2] - eye[2],
        ];
        let forward = normalize_vec3(forward);
        let right = normalize_vec3(cross_vec3(&forward, &[0.0, 1.0, 0.0]));
        let up = normalize_vec3(cross_vec3(&right, &forward));

        let scale = self.distance * (self.fov_y * 0.5).tan() * 2.0 / viewport_height.max(1.0);
        for i in 0..3 {
            self.target[i] += right[i] * delta_x * scale - up[i] * delta_y * scale;
        }
    }
}

/// Mouse interaction handle that keeps event closures alive.
pub struct OrbitControls {
    _canvas: HtmlCanvasElement,
    _camera: Rc<RefCell<Camera>>,
    _closures: Vec<JsValue>,
}

impl OrbitControls {
    /// Attaches orbit mouse listeners to the canvas.
    ///
    /// `request_render` is called whenever the camera changes.
    pub fn attach<F: FnMut() + 'static>(renderer: &Renderer, request_render: F) -> Self {
        let camera = Rc::clone(&renderer.camera);
        let canvas = renderer.canvas.clone();
        let dragging = Rc::new(RefCell::new(false));
        let panning = Rc::new(RefCell::new(false));
        let last_x = Rc::new(RefCell::new(0.0_f32));
        let last_y = Rc::new(RefCell::new(0.0_f32));

        let request_render = Rc::new(RefCell::new(request_render));

        let on_mouse_down = {
            let dragging = Rc::clone(&dragging);
            let panning = Rc::clone(&panning);
            let last_x = Rc::clone(&last_x);
            let last_y = Rc::clone(&last_y);
            Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
                *dragging.borrow_mut() = true;
                *panning.borrow_mut() = ev.button() == 1 || (ev.button() == 0 && ev.shift_key());
                *last_x.borrow_mut() = ev.client_x() as f32;
                *last_y.borrow_mut() = ev.client_y() as f32;
            })
            .into_js_value()
        };

        let on_mouse_move = {
            let camera = Rc::clone(&camera);
            let dragging = Rc::clone(&dragging);
            let panning = Rc::clone(&panning);
            let last_x = Rc::clone(&last_x);
            let last_y = Rc::clone(&last_y);
            let request_render = Rc::clone(&request_render);
            let canvas_height = canvas.client_height() as f32;
            Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
                if !*dragging.borrow() {
                    return;
                }
                let x = ev.client_x() as f32;
                let y = ev.client_y() as f32;
                let dx = x - *last_x.borrow();
                let dy = y - *last_y.borrow();
                {
                    let mut camera = camera.borrow_mut();
                    if *panning.borrow() {
                        camera.pan(-dx, -dy, canvas_height);
                    } else {
                        camera.orbit(-dx, -dy);
                    }
                }
                *last_x.borrow_mut() = x;
                *last_y.borrow_mut() = y;
                (request_render.borrow_mut())();
            })
            .into_js_value()
        };

        let on_mouse_up = {
            let dragging = Rc::clone(&dragging);
            Closure::<dyn FnMut(MouseEvent)>::new(move |_ev: MouseEvent| {
                *dragging.borrow_mut() = false;
            })
            .into_js_value()
        };

        let on_wheel = {
            let camera = Rc::clone(&camera);
            let request_render = Rc::clone(&request_render);
            Closure::<dyn FnMut(WheelEvent)>::new(move |ev: WheelEvent| {
                ev.prevent_default();
                camera.borrow_mut().zoom(ev.delta_y() as f32);
                (request_render.borrow_mut())();
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
            _camera: camera,
            _closures: vec![on_mouse_down, on_mouse_move, on_mouse_up, on_wheel],
        }
    }
}

/// `three-d` renderer for a parsed GLB document.
pub struct Renderer {
    context: ThreeDContext,
    camera: Rc<RefCell<Camera>>,
    canvas: HtmlCanvasElement,
    scene_bounds: ([f32; 3], [f32; 3]),
    models: Vec<Box<dyn Object>>,
    total_vertices: usize,
    total_triangles: usize,
    light: DirectionalLight,
}

impl Renderer {
    /// Creates a renderer for the given canvas and glTF document.
    pub fn new(canvas: HtmlCanvasElement, gltf: &Gltf) -> Option<Self> {
        let gl_context = canvas
            .get_context("webgl2")
            .ok()??
            .dyn_into::<WebGl2RenderingContext>()
            .ok()?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = gl_context;
            let _ = gltf;
            return None;
        }

        #[cfg(target_arch = "wasm32")]
        {
            let glow_context = glow::Context::from_webgl2_context(gl_context);
            #[allow(clippy::arc_with_non_send_sync)]
            let context = ThreeDContext::from_gl_context(Arc::new(glow_context)).ok()?;

            let scene_bounds = compute_bounding_box(gltf);
            let camera = Rc::new(RefCell::new(Camera::framing_bounding_box(
                scene_bounds.0,
                scene_bounds.1,
            )));

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
                canvas,
                scene_bounds,
                models,
                total_vertices,
                total_triangles,
                light,
            })
        }
    }

    /// Renders the scene once.
    pub fn render(&self) {
        let width = self.canvas.client_width() as u32;
        let height = self.canvas.client_height() as u32;
        if width == 0 || height == 0 {
            return;
        }
        self.resize();

        let camera = self.camera.borrow();
        let viewport = Viewport::new_at_origo(width, height);
        let eye = camera.eye();
        let three_d_camera = ThreeDCamera::new_perspective(
            viewport,
            vec3(eye[0], eye[1], eye[2]),
            vec3(camera.target[0], camera.target[1], camera.target[2]),
            vec3(0.0_f32, 1.0, 0.0),
            radians(camera.fov_y),
            camera.near,
            camera.far,
        );

        let objects: Vec<&dyn Object> = self
            .models
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();
        RenderTarget::screen(&self.context, width, height)
            .clear(ClearState::color_and_depth(0.05, 0.05, 0.05, 1.0, 1.0))
            .render(&three_d_camera, objects, &[&self.light]);
    }

    /// Returns a shared handle to the camera.
    pub fn camera(&self) -> Rc<RefCell<Camera>> {
        Rc::clone(&self.camera)
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
        let width = self.canvas.client_width() as u32;
        let height = self.canvas.client_height() as u32;
        if self.canvas.width() != width || self.canvas.height() != height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
        }
    }
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
