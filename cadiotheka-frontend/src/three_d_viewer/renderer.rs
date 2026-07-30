//! `three-d` renderer for a parsed GLB document.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::three_d_viewer::scene::{build_framing_camera, build_ground_grid, canvas_size};
use crate::three_d_viewer::state::{ViewState, ViewerTheme};
use leptos::web_sys::HtmlCanvasElement;
use leptos::web_sys::WebGl2RenderingContext;
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
use three_d::InnerSpace;
use three_d::MetricSpace;
use three_d::core::{ClearState, Context as ThreeDContext, RenderTarget};
use three_d::renderer::Camera as ThreeDCamera;
use three_d::renderer::DirectionalLight;
use three_d::renderer::Object;
use three_d::renderer::control::{Event, OrbitControl};
use three_d_asset::vec3;
#[cfg(target_arch = "wasm32")]
use three_d_asset::{Model, Scene, Srgba};
use wasm_bindgen::JsCast;

/// `three-d` renderer for a parsed GLB document.
pub struct Renderer {
    pub(crate) context: ThreeDContext,
    pub(crate) camera: ThreeDCamera,
    pub(crate) control: OrbitControl,
    pub(crate) canvas: HtmlCanvasElement,
    pub(crate) scene_bounds: ([f32; 3], [f32; 3]),
    pub(crate) models: Vec<Box<dyn Object>>,
    pub(crate) ground_grid: Option<Box<dyn Object>>,
    pub(crate) total_vertices: usize,
    pub(crate) total_triangles: usize,
    pub(crate) light: DirectionalLight,
    pub(crate) pending_events: Rc<RefCell<Vec<Event>>>,
    pub(crate) theme: ViewerTheme,
}

impl Renderer {
    /// Creates a renderer for the given canvas and GLB bytes.
    ///
    /// Returns `None` if WebGL2 is unavailable or the model cannot be loaded.
    pub fn new(canvas: &HtmlCanvasElement, glb_bytes: &[u8]) -> Option<Self> {
        let gl_context = canvas
            .get_context("webgl2")
            .ok()??
            .dyn_into::<WebGl2RenderingContext>()
            .ok()?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = gl_context;
            let _ = glb_bytes;
            None
        }

        #[cfg(target_arch = "wasm32")]
        {
            use crate::three_d_viewer::scene::{
                scene_bounds_from_model, suppress_webgl_debug_renderer_info,
            };
            use crate::three_d_viewer::upload::upload_primitive;

            suppress_webgl_debug_renderer_info(&gl_context);
            let glow_context = glow::Context::from_webgl2_context(gl_context);
            #[allow(clippy::arc_with_non_send_sync)]
            let context = ThreeDContext::from_gl_context(Arc::new(glow_context)).ok()?;

            let mut raw_assets = three_d_asset::io::RawAssets::new();
            raw_assets.insert("model.glb", glb_bytes.to_vec());
            let scene: Scene = raw_assets.deserialize("model.glb").ok()?;
            let model = Model::from(scene);

            let scene_bounds = scene_bounds_from_model(&model);
            let (camera, control) = build_framing_camera(scene_bounds.0, scene_bounds.1, canvas);

            let mut models = Vec::new();
            let mut total_vertices = 0;
            let mut total_triangles = 0;

            for primitive in &model.geometries {
                upload_primitive(
                    &context,
                    primitive,
                    &model.materials,
                    &mut models,
                    &mut total_vertices,
                    &mut total_triangles,
                );
            }

            let light =
                DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.3_f32, -0.8, -0.5));

            let ground_grid = Some(build_ground_grid(
                &context,
                scene_bounds.0,
                scene_bounds.1,
                ViewerTheme::default(),
            ));

            Some(Self {
                context,
                camera,
                control,
                canvas: canvas.clone(),
                scene_bounds,
                models,
                ground_grid,
                total_vertices,
                total_triangles,
                light,
                pending_events: Rc::new(RefCell::new(Vec::new())),
                theme: ViewerTheme::default(),
            })
        }
    }

    /// Serializes the current camera and theme state.
    pub fn save_view_state(&self) -> ViewState {
        let eye = self.camera.position();
        let target = self.camera.target();
        let up = self.camera.up_orthogonal();
        ViewState {
            eye: [eye.x, eye.y, eye.z],
            target: [target.x, target.y, target.z],
            up: [up.x, up.y, up.z],
            theme: self.theme,
        }
    }

    /// Restores the camera and theme from a saved state.
    pub fn restore_view_state(&mut self, state: &ViewState) {
        let viewport =
            three_d_asset::Viewport::new_at_origo(self.canvas.width(), self.canvas.height());
        self.camera = ThreeDCamera::new_perspective(
            viewport,
            vec3(state.eye[0], state.eye[1], state.eye[2]),
            vec3(state.target[0], state.target[1], state.target[2]),
            vec3(state.up[0], state.up[1], state.up[2]),
            three_d_asset::radians(Self::FOV_Y),
            self.camera.z_near(),
            self.camera.z_far(),
        );
        self.control = OrbitControl::new(
            vec3(state.target[0], state.target[1], state.target[2]),
            self.camera.z_near(),
            self.camera.z_far(),
        );
        self.set_theme(state.theme);
    }

    /// Resets the camera and orbit target to frame the loaded model.
    pub fn reset_view(&mut self) {
        let (camera, control) =
            build_framing_camera(self.scene_bounds.0, self.scene_bounds.1, &self.canvas);
        self.camera = camera;
        self.control = control;
        self.rebuild_ground_grid();
    }

    fn rebuild_ground_grid(&mut self) {
        self.ground_grid = Some(build_ground_grid(
            &self.context,
            self.scene_bounds.0,
            self.scene_bounds.1,
            self.theme,
        ));
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

        let viewport = three_d_asset::Viewport::new_at_origo(width, height);
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
            .chain(self.ground_grid.iter().map(AsRef::as_ref))
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
        target.render(&self.camera, objects, &[&self.light]);
    }

    /// Sets the viewer theme and re-renders on the next frame.
    pub fn set_theme(&mut self, theme: ViewerTheme) {
        if self.theme != theme {
            self.theme = theme;
            self.rebuild_ground_grid();
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
    pub(crate) fn pending_events(&self) -> Rc<RefCell<Vec<Event>>> {
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
