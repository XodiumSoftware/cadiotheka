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
/// the closest visible model intersection.
///
/// Hidden primitives are skipped so they do not block picking of objects behind
/// them.
///
/// `x` and `y` are in physical pixels with the origin at the bottom-left of
/// the canvas, matching `three-d`'s coordinate convention.
#[cfg(target_arch = "wasm32")]
pub fn raycast(renderer: &Renderer, x: f32, y: f32) -> Option<RaycastHit> {
    use three_d::core::render_states::Cull;
    use three_d::renderer::pick;

    let visible: Vec<(usize, &dyn three_d::renderer::Object)> = renderer
        .models
        .iter()
        .enumerate()
        .filter(|(index, _)| !renderer.is_hidden(*index))
        .map(|(index, model)| (index, model.as_ref()))
        .collect();
    if visible.is_empty() {
        return None;
    }

    let pixel = three_d_asset::PixelPoint { x, y };
    let geometries = visible.iter().map(|(_, object)| *object);
    match pick(
        &renderer.context,
        &renderer.camera,
        pixel,
        geometries,
        Cull::None,
    ) {
        Ok(Some(result)) => {
            let geometry_id = usize::try_from(result.geometry_id).unwrap_or(0);
            visible
                .get(geometry_id)
                .map(|(original_index, _)| RaycastHit {
                    primitive_index: *original_index,
                    position: crate::utils::math::vec3_to_array(result.position),
                })
        }
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
