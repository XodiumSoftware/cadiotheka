//! Simple environment lighting for the 3D IFC viewer.
//!
//! The previous implementation used a procedural cube-map skybox and image-based
//! ambient lighting from `three-d`. That path triggered repeated Firefox WebGL
//! warnings about lazy cube-map initialization because `three-d` 0.19 calls
//! `generateMipmap` immediately after allocating empty cube-map storage. Until
//! `three-d` is fixed or forked, this module provides a uniform ambient light
//! plus a directional light so the viewer renders without cube-map warnings.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use three_d::core::Context as ThreeDContext;
use three_d::renderer::AmbientLight;
use three_d_asset::Srgba;

/// Builds a uniform ambient light for the viewer.
pub fn build_ambient(context: &ThreeDContext) -> AmbientLight {
    AmbientLight::new(context, 0.6, Srgba::WHITE)
}

/// Returns a default directional light direction for the viewer.
pub fn light_direction() -> [f32; 3] {
    let direction = [0.3_f32, -0.8, -0.5];
    let length =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    [
        direction[0] / length,
        direction[1] / length,
        direction[2] / length,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_direction_is_normalized_length() {
        let [x, y, z] = light_direction();
        let length = (x * x + y * y + z * z).sqrt();
        assert!((length - 1.0).abs() < 0.01);
    }
}
