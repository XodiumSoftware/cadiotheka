//! Procedural HDR environment (skybox) for image-based lighting.
//!
//! A CPU-side cube map is generated in code and uploaded directly to the GPU.
//! This avoids `TextureCubeMap::new_from_equirectangular`, whose internal
//! render-to-cube-map path can trigger browser warnings about lazy texture
//! initialization when `generateMipmap` is called.
//!
//! A matching [`Skybox`] object can be rendered as the background and an
//! [`AmbientLight`] with the same environment provides image-based lighting.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use std::sync::Arc;
use three_d::InnerSpace;
use three_d::core::{Context as ThreeDContext, CpuTexture, Mipmap, TextureCubeMap, Wrapping};
use three_d::renderer::AmbientLight;
use three_d::renderer::Skybox;
use three_d_asset::{Srgba, TextureData, Vec3, vec3};

/// Size of each face of the generated cube map in pixels.
const CUBE_SIZE: u32 = 128;
/// HDR intensity of the horizon/sun region, above white so PBR materials pick
/// up strong specular highlights.
const HORIZON_INTENSITY: f32 = 2.2;

/// Computes an HDR RGB color for a sky direction given by a world-space unit
/// direction vector.
fn sky_color_for_direction(direction: Vec3) -> [f32; 3] {
    let direction = direction.normalize();
    let elevation = direction.y.asin();
    let azimuth = direction.z.atan2(direction.x);
    sky_color(elevation, azimuth)
}

/// Computes an HDR RGB color for a sky direction given by equirectangular
/// angles (elevation in `[-π/2, π/2]`, azimuth in `[0, 2π]`).
fn sky_color(elevation: f32, azimuth: f32) -> [f32; 3] {
    let zenith = elevation.sin().clamp(0.0, 1.0);
    let horizon = (1.0 - zenith * zenith).sqrt();

    // Blue sky with a warm band near the horizon and a cooler zenith.
    let sky = vec3(0.25, 0.55, 1.0).normalize();
    let horizon_tint = vec3(1.0, 0.85, 0.6);
    let base = sky * (0.25 + 0.55 * horizon) + horizon_tint * (0.18 * horizon);

    // A soft warm "sun" highlight whose direction rotates with the azimuth so
    // metallic surfaces receive a directional specular reflection.
    let sun_direction = vec3(azimuth.cos() * 0.6, 0.25, -azimuth.sin() * 0.6);
    let intensity = sun_direction.dot(sky).max(0.0).powi(24) * HORIZON_INTENSITY;
    let color = base * (0.35 + 0.65 * zenith) + vec3(1.0, 0.9, 0.7) * intensity;

    // Clamp the low end but keep HDR peaks above 1.0.
    [
        color.x.clamp(0.0, 4.0),
        color.y.clamp(0.0, 4.0),
        color.z.clamp(0.0, 4.0),
    ]
}

/// Generates one face of the procedural cube map.
///
/// `right` and `up` are orthonormal basis vectors for the face in world space,
/// and `direction` points toward the center of the face.
#[allow(clippy::many_single_char_names)]
fn generate_face(size: u32, direction: Vec3, up: Vec3) -> Vec<[f32; 4]> {
    let right = direction.cross(up);
    let mut data = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        let v = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;
        for x in 0..size {
            let u = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let sample_dir = (direction + right * u + up * v).normalize();
            let [r, g, b] = sky_color_for_direction(sample_dir);
            data.push([r, g, b, 1.0]);
        }
    }
    data
}

/// Creates a CPU-side cube-map face texture with the given RGBA float data.
fn cpu_face(name: &str, size: u32, data: Vec<[f32; 4]>) -> CpuTexture {
    CpuTexture {
        name: name.to_owned(),
        data: TextureData::RgbaF32(data),
        width: size,
        height: size,
        wrap_s: Wrapping::ClampToEdge,
        wrap_t: Wrapping::ClampToEdge,
        mipmap: Some(Mipmap::default()),
        ..Default::default()
    }
}

/// Builds a GPU cube map from the procedurally generated sky.
fn build_environment(context: &ThreeDContext) -> TextureCubeMap {
    let size = CUBE_SIZE;
    let right = generate_face(size, vec3(1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0));
    let left = generate_face(size, vec3(-1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0));
    let top = generate_face(size, vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0));
    let bottom = generate_face(size, vec3(0.0, -1.0, 0.0), vec3(0.0, 0.0, -1.0));
    let front = generate_face(size, vec3(0.0, 0.0, 1.0), vec3(0.0, -1.0, 0.0));
    let back = generate_face(size, vec3(0.0, 0.0, -1.0), vec3(0.0, -1.0, 0.0));
    TextureCubeMap::new(
        context,
        &cpu_face("sky-right", size, right),
        &cpu_face("sky-left", size, left),
        &cpu_face("sky-top", size, top),
        &cpu_face("sky-bottom", size, bottom),
        &cpu_face("sky-front", size, front),
        &cpu_face("sky-back", size, back),
    )
}

/// Builds an ambient light driven by the generated HDR environment.
pub fn build_ibl_ambient(context: &ThreeDContext) -> AmbientLight {
    let cube_map = build_environment(context);
    AmbientLight::new_with_environment(context, 1.0, Srgba::WHITE, &cube_map)
}

/// Builds a background skybox object from the generated HDR sky.
pub fn build_skybox(context: &ThreeDContext) -> Skybox {
    #[allow(clippy::arc_with_non_send_sync)]
    let cube_map = Arc::new(build_environment(context));
    Skybox::new_with_texture(context, cube_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_color_is_hdr_and_clamped() {
        for elevation in [
            -std::f32::consts::FRAC_PI_2,
            0.0,
            std::f32::consts::FRAC_PI_2,
        ] {
            for azimuth in [0.0_f32, std::f32::consts::FRAC_PI_2, std::f32::consts::PI] {
                let [r, g, b] = sky_color(elevation, azimuth);
                assert!((0.0..=4.0).contains(&r));
                assert!((0.0..=4.0).contains(&g));
                assert!((0.0..=4.0).contains(&b));
            }
        }
    }

    #[test]
    fn cube_faces_are_full_alpha_rgba_f32() {
        let face = generate_face(32, vec3(1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0));
        assert_eq!(face.len(), 32 * 32);
        assert!(face.iter().all(|pixel| (0.0..=4.0).contains(&pixel[0])
            && (0.0..=4.0).contains(&pixel[1])
            && (0.0..=4.0).contains(&pixel[2])
            && (pixel[3] - 1.0).abs() < f32::EPSILON));
    }

    #[test]
    fn horizon_is_brighter_than_zenith() {
        let top = sky_color(std::f32::consts::FRAC_PI_2, 0.0);
        let horizon = sky_color(0.0, 0.0);
        let top_luma = top[0] + top[1] + top[2];
        let horizon_luma = horizon[0] + horizon[1] + horizon[2];
        assert!(horizon_luma > top_luma);
    }
}
