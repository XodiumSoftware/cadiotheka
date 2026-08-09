//! Raycasting against the uploaded model primitives.
//!
//! Uses `three-d`'s `pick` function to find the closest intersection between a
//! camera ray and the scene geometry.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::three_d_viewer::renderer::Renderer;

/// Result of a successful raycast into the scene.
#[derive(Clone, Copy, Debug)]
pub struct RaycastHit {
    /// Index of the intersected primitive in the renderer's model list.
    pub primitive_index: usize,
    /// World-space position of the intersection point.
    pub position: [f32; 3],
}

/// Casts a ray from the camera through the given viewport pixel and returns
/// the closest model intersection.
///
/// `x` and `y` are in physical pixels with the origin at the bottom-left of
/// the canvas, matching `three-d`'s coordinate convention.
#[cfg(target_arch = "wasm32")]
pub fn raycast(renderer: &Renderer, x: f32, y: f32) -> Option<RaycastHit> {
    use three_d::core::render_states::Cull;
    use three_d::renderer::pick;

    let pixel = three_d_asset::PixelPoint { x, y };
    let geometries = renderer.models.iter().map(std::convert::AsRef::as_ref);
    match pick(
        &renderer.context,
        &renderer.camera,
        pixel,
        geometries,
        Cull::None,
    ) {
        Ok(Some(result)) => Some(RaycastHit {
            primitive_index: result.geometry_id as usize,
            position: crate::utils::math::vec3_to_array(result.position),
        }),
        Ok(None) => None,
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Raycast failed: {err:?}").into());
            None
        }
    }
}

/// Non-WASM stub: raycasting is not supported outside the browser.
#[cfg(not(target_arch = "wasm32"))]
pub fn raycast(_renderer: &Renderer, _x: f32, _y: f32) -> Option<RaycastHit> {
    None
}
