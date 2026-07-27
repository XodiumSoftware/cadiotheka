//! Minimal WebGL renderer for GLB models produced from IFC files.

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::pedantic
)]

use crate::utils::glb::{
    GltfDocument, compute_bounding_box, look_at_matrix, mat4_identity, mat4_mul, mat4_mul_vec3,
    mat4_to_array, mat4_to_normal_matrix_3x3, node_transform, perspective_matrix,
    read_index_accessor, read_vec3_accessor,
};
use leptos::web_sys::{
    HtmlCanvasElement, MouseEvent, WebGlBuffer, WebGlProgram, WebGlRenderingContext as Gl,
    WebGlShader, WebGlUniformLocation, WheelEvent,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;

const VERTEX_SHADER: &str = r"
attribute vec3 a_position;
attribute vec3 a_normal;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform mat3 u_normal_matrix;

varying vec3 v_normal;
varying vec3 v_view_position;

void main() {
    vec4 world_position = u_model_matrix * vec4(a_position, 1.0);
    vec4 view_position = u_view_matrix * world_position;
    v_view_position = view_position.xyz;
    v_normal = u_normal_matrix * a_normal;
    gl_Position = u_projection_matrix * view_position;
}
";

const FRAGMENT_SHADER: &str = r"
precision mediump float;

varying vec3 v_normal;
varying vec3 v_view_position;

uniform vec4 u_color;
uniform vec3 u_light_direction;
uniform vec3 u_ambient_color;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 light_dir = normalize(u_light_direction);
    float diffuse = max(dot(normal, -light_dir), 0.0);
    vec3 color = u_ambient_color * u_color.rgb + u_color.rgb * diffuse * 0.7;
    gl_FragColor = vec4(color, u_color.a);
}
";

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
        let distance = max_size * 1.5;

        Self {
            target: center,
            distance,
            yaw: std::f32::consts::PI * 0.25,
            pitch: std::f32::consts::PI * 0.15,
            fov_y: std::f32::consts::PI * 0.25,
            near: distance * 0.001,
            far: distance * 100.0,
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

    /// View matrix for the current pose.
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        look_at_matrix(self.eye(), self.target, [0.0, 1.0, 0.0])
    }

    /// Projection matrix for the given viewport aspect ratio.
    pub fn projection_matrix(&self, aspect: f32) -> [f32; 16] {
        perspective_matrix(self.fov_y, aspect, self.near, self.far)
    }

    /// Rotates around the target from a drag delta.
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += delta_x * 0.01;
        self.pitch += delta_y * 0.01;
        self.pitch = self
            .pitch
            .clamp(-std::f32::consts::PI * 0.49, std::f32::consts::PI * 0.49);
    }

    /// Zooms by changing the distance.
    pub fn zoom(&mut self, delta: f32) {
        let factor = 1.0 + delta * 0.001;
        self.distance *= factor.clamp(0.8, 1.25);
        self.distance = self.distance.max(0.01);
    }

    /// Pans the target in the camera plane.
    pub fn pan(&mut self, delta_x: f32, delta_y: f32, viewport_height: f32) {
        let eye = self.eye();
        let mut forward = [
            self.target[0] - eye[0],
            self.target[1] - eye[1],
            self.target[2] - eye[2],
        ];
        normalize(&mut forward);
        let mut right = cross(&forward, &[0.0, 1.0, 0.0]);
        normalize(&mut right);
        let mut up = cross(&right, &forward);
        normalize(&mut up);

        let scale = self.distance * (self.fov_y * 0.5).tan() * 2.0 / viewport_height.max(1.0);
        for i in 0..3 {
            self.target[i] += right[i] * delta_x * scale - up[i] * delta_y * scale;
        }
    }
}

fn normalize(v: &mut [f32; 3]) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Single uploaded primitive ready for drawing.
struct RenderPrimitive {
    position_buffer: WebGlBuffer,
    normal_buffer: WebGlBuffer,
    index_buffer: Option<WebGlBuffer>,
    index_count: Option<i32>,
    vertex_count: i32,
    color: [f32; 4],
}

/// WebGL renderer for a parsed GLB document.
pub struct Renderer {
    gl: Gl,
    program: WebGlProgram,
    a_position: u32,
    a_normal: u32,
    u_model_matrix: Option<WebGlUniformLocation>,
    u_view_matrix: Option<WebGlUniformLocation>,
    u_projection_matrix: Option<WebGlUniformLocation>,
    u_normal_matrix: Option<WebGlUniformLocation>,
    u_color: Option<WebGlUniformLocation>,
    u_light_direction: Option<WebGlUniformLocation>,
    u_ambient_color: Option<WebGlUniformLocation>,
    primitives: Vec<RenderPrimitive>,
    camera: Rc<RefCell<Camera>>,
    canvas: HtmlCanvasElement,
}

impl Renderer {
    /// Creates a renderer for the given canvas and GLB document.
    pub fn new(canvas: HtmlCanvasElement, doc: &GltfDocument) -> Option<Self> {
        let gl = canvas.get_context("webgl").ok()??.dyn_into::<Gl>().ok()?;

        let program = create_program(&gl, VERTEX_SHADER, FRAGMENT_SHADER)?;
        let a_position = gl.get_attrib_location(&program, "a_position") as u32;
        let a_normal = gl.get_attrib_location(&program, "a_normal") as u32;

        let u_model_matrix = gl.get_uniform_location(&program, "u_model_matrix");
        let u_view_matrix = gl.get_uniform_location(&program, "u_view_matrix");
        let u_projection_matrix = gl.get_uniform_location(&program, "u_projection_matrix");
        let u_normal_matrix = gl.get_uniform_location(&program, "u_normal_matrix");
        let u_color = gl.get_uniform_location(&program, "u_color");
        let u_light_direction = gl.get_uniform_location(&program, "u_light_direction");
        let u_ambient_color = gl.get_uniform_location(&program, "u_ambient_color");

        gl.enable(Gl::DEPTH_TEST);
        gl.depth_func(Gl::LEQUAL);
        gl.enable(Gl::CULL_FACE);
        gl.cull_face(Gl::BACK);
        gl.clear_color(0.05, 0.05, 0.05, 1.0);

        let (min, max) = compute_bounding_box(doc);
        let camera = Rc::new(RefCell::new(Camera::framing_bounding_box(min, max)));

        let mut renderer = Self {
            gl,
            program,
            a_position,
            a_normal,
            u_model_matrix,
            u_view_matrix,
            u_projection_matrix,
            u_normal_matrix,
            u_color,
            u_light_direction,
            u_ambient_color,
            primitives: Vec::new(),
            camera,
            canvas,
        };

        renderer.upload_document(doc);
        Some(renderer)
    }

    /// Uploads all primitives from the GLB document into GPU buffers.
    fn upload_document(&mut self, doc: &GltfDocument) {
        let identity = mat4_identity();
        for node_index in 0..doc.nodes.len() {
            self.upload_node(doc, node_index, &identity);
        }
    }

    fn upload_node(
        &mut self,
        doc: &GltfDocument,
        node_index: usize,
        parent_transform: &[[f32; 4]; 4],
    ) {
        let Some(node) = doc.nodes.get(node_index) else {
            return;
        };
        let transform = mat4_mul(parent_transform, &node_transform(node));

        if let Some(mesh_index) = node.mesh
            && let Some(mesh) = doc.meshes.get(mesh_index)
        {
            for primitive in &mesh.primitives {
                self.upload_primitive(doc, primitive, &transform);
            }
        }

        for &child in &node.children {
            self.upload_node(doc, child, &transform);
        }
    }

    fn upload_primitive(
        &mut self,
        doc: &GltfDocument,
        primitive: &crate::utils::glb::GltfPrimitive,
        transform: &[[f32; 4]; 4],
    ) {
        let Some(&position_index) = primitive.attributes.get("POSITION") else {
            return;
        };
        let Ok(positions) = read_vec3_accessor(doc, position_index) else {
            return;
        };

        let normals = primitive
            .attributes
            .get("NORMAL")
            .and_then(|&idx| read_vec3_accessor(doc, idx).ok())
            .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

        let transformed_positions: Vec<[f32; 3]> = positions
            .into_iter()
            .map(|p| mat4_mul_vec3(transform, p))
            .collect();

        let position_buffer = create_vec3_buffer(&self.gl, &flatten_vec3(&transformed_positions));
        let normal_buffer = create_vec3_buffer(&self.gl, &flatten_vec3(&normals));

        let Some(position_buffer) = position_buffer else {
            return;
        };
        let Some(normal_buffer) = normal_buffer else {
            return;
        };

        let (index_buffer, index_count, vertex_count) =
            if let Some(indices_index) = primitive.indices {
                match read_index_accessor(doc, indices_index) {
                    Ok(indices) => {
                        let count = indices.len() as i32;
                        (create_index_buffer(&self.gl, &indices), Some(count), count)
                    }
                    Err(_) => (None, None, transformed_positions.len() as i32),
                }
            } else {
                (None, None, transformed_positions.len() as i32)
            };

        let color = primitive
            .material
            .and_then(|idx| doc.materials.get(idx))
            .map_or([0.7, 0.7, 0.7, 1.0], |m| m.base_color_factor);

        self.primitives.push(RenderPrimitive {
            position_buffer,
            normal_buffer,
            index_buffer,
            index_count,
            vertex_count,
            color,
        });
    }
}

fn flatten_vec3(data: &[[f32; 3]]) -> Vec<f32> {
    let mut result = Vec::with_capacity(data.len() * 3);
    for v in data {
        result.extend_from_slice(v);
    }
    result
}

impl Renderer {
    /// Renders the scene once.
    pub fn render(&self) {
        let width = self.canvas.client_width() as u32;
        let height = self.canvas.client_height() as u32;
        if width == 0 || height == 0 {
            return;
        }
        self.resize();
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);

        let camera = self.camera.borrow();
        let aspect = width as f32 / height as f32;
        let view = camera.view_matrix();
        let projection = camera.projection_matrix(aspect);

        self.gl.use_program(Some(&self.program));

        if let Some(loc) = &self.u_light_direction {
            self.gl.uniform3f(Some(loc), 0.3, -0.8, -0.5);
        }
        if let Some(loc) = &self.u_ambient_color {
            self.gl.uniform3f(Some(loc), 0.25, 0.25, 0.25);
        }

        self.gl.uniform_matrix4fv_with_f32_array(
            self.u_view_matrix.as_ref(),
            false,
            &mat4_to_array(&view),
        );
        self.gl.uniform_matrix4fv_with_f32_array(
            self.u_projection_matrix.as_ref(),
            false,
            &projection,
        );

        let identity = mat4_identity();
        let normal_identity = mat4_to_normal_matrix_3x3(&identity);
        self.gl.uniform_matrix4fv_with_f32_array(
            self.u_model_matrix.as_ref(),
            false,
            &mat4_to_array(&identity),
        );
        self.gl.uniform_matrix3fv_with_f32_array(
            self.u_normal_matrix.as_ref(),
            false,
            &normal_identity,
        );

        for primitive in &self.primitives {
            self.draw_primitive(primitive);
        }
    }

    fn draw_primitive(&self, primitive: &RenderPrimitive) {
        self.gl
            .bind_buffer(Gl::ARRAY_BUFFER, Some(&primitive.position_buffer));
        self.gl
            .vertex_attrib_pointer_with_i32(self.a_position, 3, Gl::FLOAT, false, 0, 0);
        self.gl.enable_vertex_attrib_array(self.a_position);

        self.gl
            .bind_buffer(Gl::ARRAY_BUFFER, Some(&primitive.normal_buffer));
        self.gl
            .vertex_attrib_pointer_with_i32(self.a_normal, 3, Gl::FLOAT, false, 0, 0);
        self.gl.enable_vertex_attrib_array(self.a_normal);

        if let Some(loc) = &self.u_color {
            self.gl.uniform4f(
                Some(loc),
                primitive.color[0],
                primitive.color[1],
                primitive.color[2],
                primitive.color[3],
            );
        }

        match (&primitive.index_buffer, primitive.index_count) {
            (Some(buffer), Some(count)) => {
                self.gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(buffer));
                self.gl
                    .draw_elements_with_i32(Gl::TRIANGLES, count, Gl::UNSIGNED_INT, 0);
            }
            _ => {
                self.gl
                    .draw_arrays(Gl::TRIANGLES, 0, primitive.vertex_count);
            }
        }

        self.gl.disable_vertex_attrib_array(self.a_position);
        self.gl.disable_vertex_attrib_array(self.a_normal);
    }

    /// Returns a shared handle to the camera.
    pub fn camera(&self) -> Rc<RefCell<Camera>> {
        Rc::clone(&self.camera)
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

fn create_program(gl: &Gl, vertex: &str, fragment: &str) -> Option<WebGlProgram> {
    let vertex_shader = compile_shader(gl, Gl::VERTEX_SHADER, vertex)?;
    let fragment_shader = compile_shader(gl, Gl::FRAGMENT_SHADER, fragment)?;
    let program = gl.create_program()?;
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    if !gl
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let info = gl.get_program_info_log(&program).unwrap_or_default();
        leptos::web_sys::console::error_1(&format!("Failed to link WebGL program: {info}").into());
        return None;
    }

    Some(program)
}

fn compile_shader(gl: &Gl, shader_type: u32, source: &str) -> Option<WebGlShader> {
    let shader = gl.create_shader(shader_type)?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if !gl
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let info = gl.get_shader_info_log(&shader).unwrap_or_default();
        leptos::web_sys::console::error_1(&format!("Failed to compile shader: {info}").into());
        return None;
    }

    Some(shader)
}

fn create_vec3_buffer(gl: &Gl, data: &[f32]) -> Option<WebGlBuffer> {
    let buffer = gl.create_buffer()?;
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        Gl::ARRAY_BUFFER,
        &js_sys::Float32Array::from(data),
        Gl::STATIC_DRAW,
    );
    Some(buffer)
}

fn create_index_buffer(gl: &Gl, data: &[u32]) -> Option<WebGlBuffer> {
    let buffer = gl.create_buffer()?;
    gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        Gl::ELEMENT_ARRAY_BUFFER,
        &js_sys::Uint32Array::from(data),
        Gl::STATIC_DRAW,
    );
    Some(buffer)
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
