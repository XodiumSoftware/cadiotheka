//! Uploading `three-d-asset` primitives into `three-d` GPU objects.

use three_d::core::Context as ThreeDContext;
use three_d::core::render_states::Cull;
use three_d::renderer::Mesh;
use three_d::renderer::Object;
use three_d::renderer::PhysicalMaterial;
use three_d::renderer::geometry::CpuMesh;
use three_d::renderer::material::ColorMaterial;
use three_d_asset::Geometry;
use three_d_asset::PbrMaterial;
use three_d_asset::Srgba;
use three_d_asset::material::{GeometryFunction, LightingModel, NormalDistributionFunction};

/// Uploads a single `three-d-asset` primitive into a `three-d` render object.
#[allow(dead_code)]
pub fn upload_primitive(
    context: &ThreeDContext,
    primitive: &three_d_asset::Primitive,
    materials: &[PbrMaterial],
    models: &mut Vec<Box<dyn Object>>,
    total_vertices: &mut usize,
    total_triangles: &mut usize,
) {
    let Geometry::Triangles(tri_mesh) = &primitive.geometry else {
        return;
    };

    let position_count = tri_mesh.positions.len();
    let cpu_mesh = CpuMesh {
        positions: tri_mesh.positions.clone(),
        indices: tri_mesh.indices.clone(),
        normals: tri_mesh.normals.clone(),
        tangents: tri_mesh.tangents.clone(),
        uvs: tri_mesh.uvs.clone(),
        colors: tri_mesh.colors.clone(),
    };

    let mut mesh = Mesh::new(context, &cpu_mesh);
    mesh.set_transformation(primitive.transformation);

    let mut cpu_material = primitive
        .material_index
        .and_then(|i| materials.get(i))
        .cloned()
        .unwrap_or_else(default_ifc_material);
    cpu_material.lighting_model = choose_lighting_model(&cpu_material);

    let model: Box<dyn Object> = if cpu_material.is_unlit() {
        let mut material = ColorMaterial::new(context, &cpu_material);
        material.render_states.cull = Cull::None;
        Box::new(three_d::Gm::new(mesh, material))
    } else {
        let mut material = PhysicalMaterial::new(context, &cpu_material);
        material.render_states.cull = Cull::None;
        Box::new(three_d::Gm::new(mesh, material))
    };

    models.push(model);

    let index_count = tri_mesh.indices.len().unwrap_or(0);
    *total_vertices += position_count;
    *total_triangles += triangle_count(index_count, position_count);
}

/// Returns a default material tuned for IFC-derived geometry.
#[allow(dead_code)]
fn default_ifc_material() -> PbrMaterial {
    PbrMaterial {
        name: String::new(),
        albedo: Srgba::new(217, 217, 217, 255),
        metallic: 0.0,
        roughness: 0.8,
        ..Default::default()
    }
}

/// Picks a lighting model for the given material.
///
/// GLTF assets already specify Cook-Torrance, which gives physically correct
/// PBR shading. For untextured IFC fallback geometry we keep Blinn as a cheaper
/// default. Any material with textures benefits from Cook-Torrance so the maps
/// (normal, metallic-roughness, albedo) are evaluated in a physically based
/// shader.
fn choose_lighting_model(cpu_material: &PbrMaterial) -> LightingModel {
    let has_pbr_textures = cpu_material.albedo_texture.is_some()
        || cpu_material.metallic_roughness_texture.is_some()
        || cpu_material.occlusion_metallic_roughness_texture.is_some()
        || cpu_material.normal_texture.is_some()
        || cpu_material.emissive_texture.is_some();

    if has_pbr_textures {
        LightingModel::Cook(
            NormalDistributionFunction::TrowbridgeReitzGGX,
            GeometryFunction::SmithSchlickGGX,
        )
    } else {
        LightingModel::Blinn
    }
}

/// Returns whether the given material should be rendered unlit.
#[allow(dead_code)]
trait UnlitMaterial {
    fn is_unlit(&self) -> bool;
}

#[allow(dead_code)]
impl UnlitMaterial for PbrMaterial {
    fn is_unlit(&self) -> bool {
        matches!(self.lighting_model, LightingModel::Blinn)
            && self.metallic == 0.0
            && (self.roughness - 1.0).abs() < f32::EPSILON
            && self.emissive == Srgba::BLACK
    }
}

/// Counts triangles for a primitive given its index and vertex counts.
#[allow(dead_code)]
fn triangle_count(index_count: usize, vertex_count: usize) -> usize {
    if index_count == 0 {
        vertex_count / 3
    } else {
        index_count / 3
    }
}
