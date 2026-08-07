//! Procedural HDR environment (skybox) for image-based lighting.
//!
//! An equirectangular HDR map is generated in code and converted into an IBL
//! environment used by [`AmbientLight`]. A matching [`Skybox`] object can be
//! rendered as the background.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use three_d::InnerSpace;
use three_d::core::{Context as ThreeDContext, TextureCubeMap};
use three_d::renderer::AmbientLight;
use three_d::renderer::Skybox;
use three_d_asset::{Srgba, Texture2D, TextureData, vec3};

/// Width of the generated equirectangular sky map in pixels.
const SKY_WIDTH: u32 = 512;
/// Height of the generated equirectangular sky map in pixels.
const SKY_HEIGHT: u32 = 256;
/// HDR intensity of the horizon/sun region, above white so PBR materials pick
/// up strong specular highlights.
const HORIZON_INTENSITY: f32 = 2.2;

/// Generates an equirectangular HDR sky map with a soft zenith-to-horizon
/// gradient and a subtle horizontal tint.
///
/// Returns a CPU-side [`Texture2D`] holding `RgbF32` data suitable for building
/// a cube map via [`TextureCubeMap::new_from_equirectangular`].
pub fn equirectangular_sky(width: u32, height: u32) -> Texture2D {
    let mut data = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        let v = (y as f32 + 0.5) / height as f32;
        let elevation = v * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
        for x in 0..width {
            let u = (x as f32 + 0.5) / width as f32;
            let azimuth = u * std::f32::consts::TAU;
            data.push(sky_color(elevation, azimuth));
        }
    }
    Texture2D {
        name: "procedural-sky.hdr".to_owned(),
        data: TextureData::RgbF32(data),
        width,
        height,
        ..Default::default()
    }
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

/// Builds an ambient light driven by the generated HDR environment.
pub fn build_ibl_ambient(context: &ThreeDContext) -> AmbientLight {
    let map = equirectangular_sky(SKY_WIDTH, SKY_HEIGHT);
    let cube_map = build_environment(context, &map);
    AmbientLight::new_with_environment(context, 1.0, Srgba::WHITE, &cube_map)
}

/// Converts an equirectangular HDR map into a cube map.
fn build_environment(context: &ThreeDContext, cpu_texture: &Texture2D) -> TextureCubeMap {
    TextureCubeMap::new_from_equirectangular::<f32>(context, cpu_texture)
}

/// Builds a background skybox object from the generated HDR sky.
pub fn build_skybox(context: &ThreeDContext) -> Skybox {
    let map = equirectangular_sky(SKY_WIDTH, SKY_HEIGHT);
    Skybox::new_from_equirectangular(context, &map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_map_has_expected_dimensions_and_data() {
        let map = equirectangular_sky(64, 32);
        assert_eq!(map.width, 64);
        assert_eq!(map.height, 32);
        if let TextureData::RgbF32(data) = &map.data {
            assert_eq!(data.len(), 64 * 32);
        } else {
            panic!("expected RgbF32 sky data");
        }
    }

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
    fn horizon_is_brighter_than_zenith() {
        let top = sky_color(std::f32::consts::FRAC_PI_2, 0.0);
        let horizon = sky_color(0.0, 0.0);
        let top_luma = top[0] + top[1] + top[2];
        let horizon_luma = horizon[0] + horizon[1] + horizon[2];
        assert!(horizon_luma > top_luma);
    }
}
