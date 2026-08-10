//! `three-d` renderer for a parsed GLB document.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::three_d_viewer::environment::build_skybox;
use crate::three_d_viewer::scene::{build_axes, build_framing_camera, canvas_size};
use crate::three_d_viewer::state::{ViewDirection, ViewState, ViewerTheme};
use leptos::web_sys::HtmlCanvasElement;
use leptos::web_sys::WebGl2RenderingContext;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
use three_d::InnerSpace;
use three_d::MetricSpace;
use three_d::core::{ClearState, Context as ThreeDContext, RenderTarget};
use three_d::renderer::AmbientLight;
use three_d::renderer::BoundingBox;
use three_d::renderer::Camera as ThreeDCamera;
use three_d::renderer::ColorMaterial;
use three_d::renderer::DirectionalLight;
use three_d::renderer::Gm;
use three_d::renderer::Object;
use three_d::renderer::Skybox;
use three_d::renderer::control::{Event, OrbitControl};
use three_d_asset::Srgba;
use three_d_asset::vec3;
#[cfg(target_arch = "wasm32")]
use three_d_asset::{Model, Scene};
use wasm_bindgen::JsCast;

/// `three-d` renderer for a parsed GLB document.
pub struct Renderer {
    pub(crate) context: ThreeDContext,
    pub(crate) camera: ThreeDCamera,
    pub(crate) control: OrbitControl,
    pub(crate) canvas: HtmlCanvasElement,
    pub(crate) scene_bounds: ([f32; 3], [f32; 3]),
    pub(crate) models: Vec<Box<dyn Object>>,
    pub(crate) axes: Option<Box<dyn Object>>,
    pub(crate) outline: Option<Gm<BoundingBox, ColorMaterial>>,
    pub(crate) hovered_primitive: Option<usize>,
    pub(crate) hidden_primitives: HashSet<usize>,
    pub(crate) highlight_color: Srgba,
    pub(crate) skybox_color: Srgba,
    pub(crate) skybox: Option<Skybox>,
    pub(crate) show_axes: bool,
    pub(crate) total_vertices: usize,
    pub(crate) total_triangles: usize,
    pub(crate) light: DirectionalLight,
    pub(crate) ambient: AmbientLight,
    pub(crate) pending_events: Rc<RefCell<Vec<Event>>>,
    pub(crate) theme: ViewerTheme,
}

impl Renderer {
    /// Creates a renderer for the given canvas without loading a model.
    ///
    /// The WebGL2 context, lights, and environment are created once. Use
    /// [`Self::load_model`] to load GLB data into the renderer afterwards.
    /// Returns `None` if WebGL2 is unavailable.
    pub fn new(canvas: &HtmlCanvasElement) -> Option<Self> {
        let gl_context = canvas
            .get_context("webgl2")
            .ok()??
            .dyn_into::<WebGl2RenderingContext>()
            .ok()?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = gl_context;
            None
        }

        #[cfg(target_arch = "wasm32")]
        {
            use crate::three_d_viewer::environment::build_ibl_ambient;
            use crate::three_d_viewer::scene::suppress_webgl_debug_renderer_info;

            suppress_webgl_debug_renderer_info(&gl_context);
            let glow_context = glow::Context::from_webgl2_context(gl_context);
            #[allow(clippy::arc_with_non_send_sync)]
            let context = ThreeDContext::from_gl_context(Arc::new(glow_context)).ok()?;

            let light =
                DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.3_f32, -0.8, -0.5));
            let ambient = build_ibl_ambient(&context);
            let skybox_color = Srgba::WHITE;
            let skybox = Some(build_skybox(&context, skybox_color));

            let (camera, control) = build_framing_camera([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], canvas);

            Some(Self {
                context,
                camera,
                control,
                canvas: canvas.clone(),
                scene_bounds: ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
                models: Vec::new(),
                axes: None,
                outline: None,
                hovered_primitive: None,
                hidden_primitives: HashSet::new(),
                highlight_color: Srgba::new(255, 200, 0, 255),
                skybox_color,
                skybox,
                show_axes: true,
                total_vertices: 0,
                total_triangles: 0,
                light,
                ambient,
                pending_events: Rc::new(RefCell::new(Vec::new())),
                theme: ViewerTheme::default(),
            })
        }
    }

    /// Loads a GLB model into the existing renderer.
    ///
    /// Replaces any previously loaded model, rebuilds the camera and axes to
    /// match the new scene bounds, and returns `true` on success.
    #[cfg(target_arch = "wasm32")]
    pub fn load_model(&mut self, glb_bytes: &[u8]) -> bool {
        use crate::three_d_viewer::scene::scene_bounds_from_model;
        use crate::three_d_viewer::upload::upload_primitive;

        let mut raw_assets = three_d_asset::io::RawAssets::new();
        raw_assets.insert("model.glb", glb_bytes.to_vec());
        let scene: Scene = match raw_assets.deserialize("model.glb") {
            Ok(scene) => scene,
            Err(_) => return false,
        };
        let model = Model::from(scene);

        self.scene_bounds = scene_bounds_from_model(&model);
        let (camera, control) =
            build_framing_camera(self.scene_bounds.0, self.scene_bounds.1, &self.canvas);
        self.camera = camera;
        self.control = control;

        self.models.clear();
        self.total_vertices = 0;
        self.total_triangles = 0;
        self.outline = None;
        self.hidden_primitives.clear();
        self.hovered_primitive = None;
        for primitive in &model.geometries {
            upload_primitive(
                &self.context,
                primitive,
                &model.materials,
                &mut self.models,
                &mut self.total_vertices,
                &mut self.total_triangles,
            );
        }

        self.rebuild_axes(self.show_axes);
        true
    }

    /// Non-WASM stub: loading models is not supported outside the browser.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_model(&mut self, _glb_bytes: &[u8]) -> bool {
        false
    }

    /// Creates a renderer for the given canvas and GLB bytes.
    ///
    /// This convenience method combines [`Self::new`] and [`Self::load_model`].
    /// Returns `None` if WebGL2 is unavailable or the model cannot be loaded.
    pub fn new_with_model(canvas: &HtmlCanvasElement, glb_bytes: &[u8]) -> Option<Self> {
        let mut renderer = Self::new(canvas)?;
        if renderer.load_model(glb_bytes) {
            Some(renderer)
        } else {
            None
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

    /// Restores the camera, theme, and axes visibility from a saved state.
    pub fn restore_view_state(&mut self, state: &ViewState, show_axes: bool) {
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
        self.set_show_axes(show_axes);
    }

    /// Resets the camera and orbit target to frame the loaded model.
    pub fn reset_view(&mut self, show_axes: bool) {
        let (camera, control) =
            build_framing_camera(self.scene_bounds.0, self.scene_bounds.1, &self.canvas);
        self.camera = camera;
        self.control = control;
        self.rebuild_axes(show_axes);
    }

    /// Moves the camera so it looks at the model center from the given axis direction.
    ///
    /// The camera maintains its current distance from the target (or falls back to the
    /// framing distance when the current camera is too close to the target), and the
    /// up vector is chosen so the model stays upright along the world Y axis whenever
    /// possible.
    pub fn set_focus(&mut self, direction: ViewDirection) {
        let (min, max) = self.scene_bounds;
        let center = vec3(
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        );

        let size = [
            (max[0] - min[0]).abs(),
            (max[1] - min[1]).abs(),
            (max[2] - min[2]).abs(),
        ];
        let max_size = size[0].max(size[1]).max(size[2]).max(1.0);

        let fov_y = Self::FOV_Y;
        let (width, height) = canvas_size(&self.canvas);
        let aspect = if height == 0 {
            1.0
        } else {
            width as f32 / height as f32
        };
        let half_fov_y = fov_y * 0.5;
        let fov_x = 2.0 * (aspect * half_fov_y.tan()).atan();
        let limiting_fov = fov_y.min(fov_x);
        let padding = 1.2;
        let framing_distance = (max_size * 0.5 / (limiting_fov * 0.5).tan()) * padding;

        let current_distance = self.camera.position().distance(center);
        let distance = current_distance.max(framing_distance);

        let (eye, up) = direction.eye_and_up(distance);
        let eye = center + eye;

        let viewport = three_d_asset::Viewport::new_at_origo(width, height);
        self.camera = ThreeDCamera::new_perspective(
            viewport,
            eye,
            center,
            up,
            three_d_asset::radians(fov_y),
            self.camera.z_near(),
            self.camera.z_far(),
        );
        self.control = OrbitControl::new(center, max_size * 0.001, max_size * 1_000.0);
    }

    fn rebuild_axes(&mut self, show: bool) {
        self.axes = Some(build_axes(
            &self.context,
            self.scene_bounds.0,
            self.scene_bounds.1,
        ));
        self.show_axes = show;
    }

    /// Vertical field of view in radians.
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
            .enumerate()
            .filter(|(index, _)| !self.hidden_primitives.contains(index))
            .map(|(_, model)| model.as_ref())
            .chain(
                self.axes
                    .iter()
                    .filter(|_| self.show_axes)
                    .map(AsRef::as_ref),
            )
            .chain(self.skybox.iter().flatten())
            .chain(self.outline.iter().map(|outline| outline as &dyn Object))
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
        target.render(&self.camera, objects, &[&self.light, &self.ambient]);
    }

    /// Rebuilds the skybox with the given tint color.
    ///
    /// The IBL ambient light is left unchanged, so model lighting stays
    /// consistent while only the background changes.
    pub fn set_skybox_color(&mut self, color: Srgba) {
        self.skybox_color = color;
        self.skybox = Some(build_skybox(&self.context, color));
    }

    /// Sets whether the axes gizmo is rendered.
    pub fn set_show_axes(&mut self, show: bool) {
        self.show_axes = show;
    }

    /// Resizes the canvas backing store to match its display size.
    pub fn resize(&self) {
        let (width, height) = canvas_size(&self.canvas);
        if self.canvas.width() != width || self.canvas.height() != height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
        }
    }

    /// Sets the viewer theme and re-renders on the next frame.
    pub fn set_theme(&mut self, theme: ViewerTheme) {
        if self.theme != theme {
            self.theme = theme;
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

    /// Casts a ray from the camera through the given viewport pixel and
    /// returns the closest model intersection.
    ///
    /// `x` and `y` are in physical pixels with the origin at the bottom-left
    /// of the canvas, matching `three-d`'s coordinate convention.
    pub fn pick(&self, x: f32, y: f32) -> Option<crate::three_d_viewer::raycast::RaycastHit> {
        crate::three_d_viewer::raycast::raycast(self, x, y)
    }

    /// Sets the primitive to highlight with a bounding-box outline.
    ///
    /// Pass `None` to remove the outline.
    pub fn set_hovered_primitive(&mut self, index: Option<usize>) {
        self.hovered_primitive = index;
        self.outline = index.and_then(|i| self.build_outline(i));
    }

    /// Sets the outline highlight color and rebuilds the current outline if one exists.
    pub fn set_highlight_color(&mut self, color: Srgba) {
        self.highlight_color = color;
        self.outline = self.hovered_primitive.and_then(|i| self.build_outline(i));
    }

    /// Hides the primitive with the given index so it is no longer rendered or
    /// pickable.
    pub fn hide_primitive(&mut self, index: usize) {
        if index < self.models.len() {
            self.hidden_primitives.insert(index);
            if self.hovered_primitive == Some(index) {
                self.set_hovered_primitive(None);
            }
        }
    }

    /// Shows the primitive with the given index if it was previously hidden.
    pub fn show_primitive(&mut self, index: usize) {
        self.hidden_primitives.remove(&index);
    }

    /// Shows every primitive that was hidden.
    pub fn show_all(&mut self) {
        self.hidden_primitives.clear();
    }

    /// Returns whether the primitive with the given index is currently hidden.
    pub fn is_hidden(&self, index: usize) -> bool {
        self.hidden_primitives.contains(&index)
    }

    /// Returns the number of currently hidden primitives.
    pub fn hidden_count(&self) -> usize {
        self.hidden_primitives.len()
    }

    /// Builds a bounding-box outline object around the given primitive.
    fn build_outline(
        &self,
        index: usize,
    ) -> Option<Gm<three_d::renderer::geometry::BoundingBox, ColorMaterial>> {
        let model = self.models.get(index)?;
        let aabb = model.aabb();
        let margin = aabb.size().magnitude() * 0.001_f32;
        let min = aabb.min() - vec3(margin, margin, margin);
        let max = aabb.max() + vec3(margin, margin, margin);
        let outline_aabb = three_d_asset::AxisAlignedBoundingBox::new_with_positions(&[min, max]);
        let thickness = aabb.size().magnitude() * 0.005_f32;
        let geometry =
            BoundingBox::new_with_thickness(&self.context, outline_aabb, thickness.max(0.001));
        let color = self.highlight_color;
        let cpu_material = three_d_asset::PbrMaterial {
            albedo: color,
            ..Default::default()
        };
        let material = ColorMaterial::new_opaque(&self.context, &cpu_material);
        Some(Gm::new(geometry, material))
    }
}
