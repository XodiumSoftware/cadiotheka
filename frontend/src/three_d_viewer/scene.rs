//! Scene framing, bounds, and WebGL context helpers.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::utils::math::vec3_to_array;
use leptos::web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use leptos::web_sys::WebGl2RenderingContext;
use three_d::core::Context as ThreeDContext;
use three_d::renderer::Camera as ThreeDCamera;
use three_d::renderer::Object;
use three_d::renderer::control::OrbitControl;
use three_d_asset::{Viewport, radians, vec3};

#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};

/// Returns the canvas backing store size in physical pixels.
pub fn canvas_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
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
pub fn suppress_webgl_debug_renderer_info(context: &WebGl2RenderingContext) {
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

/// Builds a perspective camera and orbit controller that frame the given bounds.
pub fn build_framing_camera(
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

/// Creates an axes gizmo at the world origin with arrow dimensions scaled to
/// the model size.
pub fn build_axes(
    context: &ThreeDContext,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
) -> Box<dyn Object> {
    let size = [
        (bounds_max[0] - bounds_min[0]).abs(),
        (bounds_max[1] - bounds_min[1]).abs(),
        (bounds_max[2] - bounds_min[2]).abs(),
    ];
    let max_size = size[0].max(size[1]).max(size[2]).max(1.0);
    let length = max_size * 0.08;
    let radius = length * 0.04;
    Box::new(three_d::renderer::Axes::new(context, radius, length))
}

/// Computes an axis-aligned world-space bounding box from a parsed model.
#[allow(dead_code)]
pub fn scene_bounds_from_model(model: &three_d_asset::Model) -> ([f32; 3], [f32; 3]) {
    use three_d_asset::Geometry;

    let mut aabb = three_d_asset::AxisAlignedBoundingBox::EMPTY;
    for primitive in &model.geometries {
        let local_aabb = match &primitive.geometry {
            Geometry::Triangles(mesh) => mesh.compute_aabb(),
            Geometry::Points(_) => three_d_asset::AxisAlignedBoundingBox::EMPTY,
        }
        .transformed(primitive.transformation);
        aabb.expand_with_aabb(local_aabb);
    }

    if aabb.is_empty() {
        ([0.0; 3], [1.0; 3])
    } else {
        (vec3_to_array(aabb.min()), vec3_to_array(aabb.max()))
    }
}
