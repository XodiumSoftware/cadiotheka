//! Minimal glTF/GLB helpers for rendering IFC models.
//!
//! This module is a thin wrapper around the `gltf` crate that exposes the
//! small API surface required by the custom WebGL renderer.

pub use gltf::Gltf;
pub use gltf::mesh::util::ReadIndices;
pub use gltf::scene::Transform;

/// Metadata attached to a glTF node by `ifc-lite-export`.
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    /// IFC express identifier from `extras.expressId`.
    pub express_id: Option<u32>,
    /// Human-readable name of the glTF node.
    pub name: String,
}

impl NodeMetadata {
    /// Extracts metadata from a glTF node.
    pub fn from_node(node: &gltf::Node<'_>) -> Self {
        let express_id = node.extras().as_ref().and_then(|raw| {
            let value: serde_json::Value = serde_json::from_str(raw.get()).ok()?;
            value
                .get("expressId")?
                .as_u64()
                .and_then(|id| id.try_into().ok())
        });
        Self {
            express_id,
            name: node.name().unwrap_or("").to_string(),
        }
    }
}

/// Builds a map from node index to its IFC metadata.
pub fn build_node_metadata_map(gltf: &Gltf) -> std::collections::HashMap<usize, NodeMetadata> {
    let mut map = std::collections::HashMap::new();
    if let Some(scene) = gltf.default_scene() {
        for node in scene.nodes() {
            visit_node_metadata(&node, &mut map);
        }
    }
    map
}

fn visit_node_metadata(
    node: &gltf::Node<'_>,
    map: &mut std::collections::HashMap<usize, NodeMetadata>,
) {
    let index = node.index();
    map.insert(index, NodeMetadata::from_node(node));
    for child in node.children() {
        visit_node_metadata(&child, map);
    }
}

/// Extracts world-space geometry for every renderable primitive in the default
/// scene, in the same order meshes are typically uploaded to the GPU.
pub fn extract_scene_geometries(gltf: &Gltf) -> Vec<crate::utils::three_d_renderer::MeshGeometry> {
    let mut geometries = Vec::new();
    if let Some(scene) = gltf.default_scene() {
        let identity = crate::utils::math::mat4_identity();
        for node in scene.nodes() {
            visit_node_geometry(gltf, &node, &identity, &mut geometries);
        }
    }
    geometries
}

fn visit_node_geometry(
    gltf: &Gltf,
    node: &gltf::Node<'_>,
    parent_transform: &[[f32; 4]; 4],
    geometries: &mut Vec<crate::utils::three_d_renderer::MeshGeometry>,
) {
    let transform = crate::utils::math::mat4_mul(parent_transform, &node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let Some(positions) = read_positions(gltf, &primitive, &transform) else {
                continue;
            };
            let indices = read_indices(gltf, &primitive);
            geometries.push(crate::utils::three_d_renderer::MeshGeometry { positions, indices });
        }
    }
    for child in node.children() {
        visit_node_geometry(gltf, &child, &transform, geometries);
    }
}

use crate::utils::math::{
    mat4_identity, mat4_mul, mat4_mul_vec3, mat4_to_normal_matrix_3x3, normalize_vec3,
};
use gltf::buffer::Source;

/// Material parameters extracted from a glTF material.
#[derive(Debug, Clone, Copy)]
pub struct GltfMaterial {
    /// Base color RGBA factor.
    pub base_color_factor: [f32; 4],
    /// Metallic factor in the range `[0.0, 1.0]`.
    pub metallic_factor: f32,
    /// Roughness factor in the range `[0.0, 1.0]`.
    pub roughness_factor: f32,
    /// Whether the material is double-sided (IFC faces are often unreliable).
    pub double_sided: bool,
    /// Whether the material should be rendered unlit (flat base color).
    pub unlit: bool,
}

impl Default for GltfMaterial {
    fn default() -> Self {
        Self {
            base_color_factor: [0.7, 0.7, 0.7, 1.0],
            metallic_factor: 0.5,
            roughness_factor: 0.5,
            double_sided: true,
            unlit: false,
        }
    }
}

impl GltfMaterial {
    /// Returns a reasonable default for IFC-derived geometry when no material
    /// is defined: light grey, slightly rough, non-metallic, double-sided.
    pub fn ifc_default() -> Self {
        Self {
            base_color_factor: [0.85, 0.85, 0.85, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.8,
            double_sided: true,
            unlit: false,
        }
    }
}

/// Resolves buffer data from the glTF document and its embedded GLB blob.
fn buffer_data<'a>(gltf: &'a Gltf, buffer: &gltf::Buffer<'a>) -> Option<&'a [u8]> {
    match buffer.source() {
        Source::Bin => gltf.blob.as_deref(),
        Source::Uri(_uri) => None,
    }
}

/// Reads the `POSITION` attribute of a primitive as world-space `[f32; 3]`
/// vectors, applying the node transform.
///
/// Returns `None` if the primitive has no positions or the accessor data is
/// unavailable.
pub fn read_positions(
    gltf: &Gltf,
    primitive: &gltf::Primitive<'_>,
    transform: &[[f32; 4]; 4],
) -> Option<Vec<[f32; 3]>> {
    let reader = primitive.reader(|buffer| buffer_data(gltf, &buffer));
    let positions = reader.read_positions()?;
    Some(positions.map(|p| mat4_mul_vec3(transform, p)).collect())
}

/// Reads the `NORMAL` attribute of a primitive as `[f32; 3]` vectors,
/// applying the node transform's normal matrix.
///
/// Falls back to a zero-length normal if normals are missing.
pub fn read_normals(
    gltf: &Gltf,
    primitive: &gltf::Primitive<'_>,
    transform: &[[f32; 4]; 4],
) -> Vec<[f32; 3]> {
    let reader = primitive.reader(|buffer| buffer_data(gltf, &buffer));
    let normal_matrix = mat4_to_normal_matrix_3x3(transform);
    if let Some(normals) = reader.read_normals() {
        normals
            .map(|n| {
                let transformed = [
                    normal_matrix[0] * n[0] + normal_matrix[3] * n[1] + normal_matrix[6] * n[2],
                    normal_matrix[1] * n[0] + normal_matrix[4] * n[1] + normal_matrix[7] * n[2],
                    normal_matrix[2] * n[0] + normal_matrix[5] * n[1] + normal_matrix[8] * n[2],
                ];
                normalize_vec3(transformed)
            })
            .collect()
    } else {
        let count = reader
            .read_positions()
            .map_or(0, std::iter::Iterator::count);
        vec![[0.0, 1.0, 0.0]; count]
    }
}

/// Reads a primitive's indices as `Vec<u32>`.
///
/// Returns `None` if the primitive is not indexed.
pub fn read_indices(gltf: &Gltf, primitive: &gltf::Primitive<'_>) -> Option<Vec<u32>> {
    let reader = primitive.reader(|buffer| buffer_data(gltf, &buffer));
    reader
        .read_indices()
        .map(|indices| indices.into_u32().collect())
}

/// Extracts material parameters from a glTF material, applying IFC-tailored
/// defaults when values are missing.
pub fn material_params(material: &gltf::Material<'_>) -> GltfMaterial {
    let pbr = material.pbr_metallic_roughness();
    let base = pbr.base_color_factor();
    GltfMaterial {
        base_color_factor: base,
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        double_sided: true,
        unlit: material.unlit(),
    }
}

/// Computes a world-space axis-aligned bounding box for the whole glTF asset.
pub fn compute_bounding_box(gltf: &Gltf) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    let identity = mat4_identity();
    if let Some(scene) = gltf.default_scene() {
        for node in scene.nodes() {
            visit_node_bounds(gltf, &node, &identity, &mut min, &mut max);
        }
    }

    if min[0].is_infinite() {
        min = [0.0; 3];
        max = [1.0; 3];
    }
    (min, max)
}

fn visit_node_bounds(
    gltf: &Gltf,
    node: &gltf::Node<'_>,
    parent_transform: &[[f32; 4]; 4],
    min: &mut [f32; 3],
    max: &mut [f32; 3],
) {
    let transform = mat4_mul(parent_transform, &node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if let Some(positions) = read_positions(gltf, &primitive, &transform) {
                for p in positions {
                    for i in 0..3 {
                        min[i] = min[i].min(p[i]);
                        max[i] = max[i].max(p[i]);
                    }
                }
            }
        }
    }

    for child in node.children() {
        visit_node_bounds(gltf, &child, &transform, min, max);
    }
}

/// Returns whether the given primitive mode describes triangles.
pub fn is_triangle_mode(mode: gltf::mesh::Mode) -> bool {
    matches!(
        mode,
        gltf::mesh::Mode::Triangles
            | gltf::mesh::Mode::TriangleStrip
            | gltf::mesh::Mode::TriangleFan
    )
}

/// Returns the number of vertices for a primitive mode given an index count.
pub fn mode_vertex_count(mode: gltf::mesh::Mode, index_count: usize) -> usize {
    match mode {
        gltf::mesh::Mode::Points
        | gltf::mesh::Mode::Lines
        | gltf::mesh::Mode::LineLoop
        | gltf::mesh::Mode::LineStrip
        | gltf::mesh::Mode::Triangles
        | gltf::mesh::Mode::TriangleStrip
        | gltf::mesh::Mode::TriangleFan => index_count,
    }
}

/// Returns the triangle count for a primitive.
pub fn triangle_count(mode: gltf::mesh::Mode, index_count: usize, vertex_count: usize) -> usize {
    match mode {
        gltf::mesh::Mode::Triangles => index_count / 3,
        gltf::mesh::Mode::TriangleStrip | gltf::mesh::Mode::TriangleFan => {
            index_count.saturating_sub(2)
        }
        _ => vertex_count / 3,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::*;
    use crate::utils::math::mat4_identity;

    fn assert_f32_array_eq<const N: usize>(actual: &[f32; N], expected: &[f32; N]) {
        for (a, b) in actual.iter().zip(expected.iter()) {
            assert!(
                (a - b).abs() < f32::EPSILON,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    /// Builds a non-indexed triangle primitive, returning the GLB bytes.
    fn build_non_indexed_glb(positions: &[[f32; 3]], translation: [f32; 3]) -> Vec<u8> {
        let mut position_bytes = Vec::new();
        for p in positions {
            for component in p {
                position_bytes.extend_from_slice(&component.to_le_bytes());
            }
        }

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }

        let json = serde_json::json!({
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "mesh": 0, "translation": translation }],
            "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "mode": 4 }] }],
            "buffers": [{ "byteLength": position_bytes.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": 0, "byteLength": position_bytes.len(), "target": 34962 }
            ],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": positions.len(), "type": "VEC3", "max": max, "min": min }
            ]
        });

        build_glb_chunks(&json, &position_bytes)
    }

    /// Builds a minimal binary GLB v2 asset containing a single triangle.
    fn build_test_glb(positions: &[[f32; 3]], indices: &[u16], translation: [f32; 3]) -> Vec<u8> {
        let mut position_bytes = Vec::new();
        for p in positions {
            for component in p {
                position_bytes.extend_from_slice(&component.to_le_bytes());
            }
        }

        let mut index_bytes = Vec::new();
        for i in indices {
            index_bytes.extend_from_slice(&i.to_le_bytes());
        }
        while !index_bytes.len().is_multiple_of(4) {
            index_bytes.push(0);
        }

        let index_offset = position_bytes.len();
        let mut bin_chunk = position_bytes.clone();
        bin_chunk.extend_from_slice(&index_bytes);

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }

        let json = serde_json::json!({
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "mesh": 0, "translation": translation }],
            "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "indices": 1, "material": 0, "mode": 4 }] }],
            "materials": [{
                "pbrMetallicRoughness": {
                    "baseColorFactor": [1.0, 0.0, 0.0, 1.0],
                    "metallicFactor": 0.1,
                    "roughnessFactor": 0.2
                },
                "doubleSided": true,
                "extensions": { "KHR_materials_unlit": {} }
            }],
            "buffers": [{ "byteLength": bin_chunk.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": 0, "byteLength": position_bytes.len(), "target": 34962 },
                { "buffer": 0, "byteOffset": index_offset, "byteLength": index_bytes.len(), "target": 34963 }
            ],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": positions.len(), "type": "VEC3", "max": max, "min": min },
                { "bufferView": 1, "componentType": 5123, "count": indices.len(), "type": "SCALAR" }
            ]
        });

        build_glb_chunks(&json, &bin_chunk)
    }

    /// Builds a single-triangle indexed GLB with per-vertex normals.
    fn build_indexed_glb_with_normals(
        positions: &[[f32; 3]],
        indices: &[u16],
        normals: &[[f32; 3]],
        translation: [f32; 3],
    ) -> Vec<u8> {
        let mut position_bytes = Vec::new();
        for p in positions {
            for component in p {
                position_bytes.extend_from_slice(&component.to_le_bytes());
            }
        }

        let mut normal_bytes = Vec::new();
        for n in normals {
            for component in n {
                normal_bytes.extend_from_slice(&component.to_le_bytes());
            }
        }

        let mut index_bytes = Vec::new();
        for i in indices {
            index_bytes.extend_from_slice(&i.to_le_bytes());
        }
        while !index_bytes.len().is_multiple_of(4) {
            index_bytes.push(0);
        }

        let normal_offset = position_bytes.len();
        let index_offset = normal_offset + normal_bytes.len();
        let mut bin_chunk = position_bytes.clone();
        bin_chunk.extend_from_slice(&normal_bytes);
        bin_chunk.extend_from_slice(&index_bytes);

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }

        let json = serde_json::json!({
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "mesh": 0, "translation": translation }],
            "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "mode": 4 }] }],
            "buffers": [{ "byteLength": bin_chunk.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": 0, "byteLength": position_bytes.len(), "target": 34962 },
                { "buffer": 0, "byteOffset": normal_offset, "byteLength": normal_bytes.len(), "target": 34962 },
                { "buffer": 0, "byteOffset": index_offset, "byteLength": index_bytes.len(), "target": 34963 }
            ],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": positions.len(), "type": "VEC3", "max": max, "min": min },
                { "bufferView": 1, "componentType": 5126, "count": normals.len(), "type": "VEC3" },
                { "bufferView": 2, "componentType": 5123, "count": indices.len(), "type": "SCALAR" }
            ]
        });

        build_glb_chunks(&json, &bin_chunk)
    }

    /// Builds a two-node GLB where the root node has `root_translation` and the
    /// child mesh node has `child_translation`.
    fn build_hierarchy_glb(
        child_positions: &[[f32; 3]],
        indices: &[u16],
        root_translation: [f32; 3],
        child_translation: [f32; 3],
    ) -> Vec<u8> {
        let mut position_bytes = Vec::new();
        for p in child_positions {
            for component in p {
                position_bytes.extend_from_slice(&component.to_le_bytes());
            }
        }

        let mut index_bytes = Vec::new();
        for i in indices {
            index_bytes.extend_from_slice(&i.to_le_bytes());
        }
        while !index_bytes.len().is_multiple_of(4) {
            index_bytes.push(0);
        }

        let index_offset = position_bytes.len();
        let mut bin_chunk = position_bytes.clone();
        bin_chunk.extend_from_slice(&index_bytes);

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in child_positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }

        let json = serde_json::json!({
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [
                { "children": [1], "translation": root_translation },
                { "mesh": 0, "translation": child_translation }
            ],
            "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "indices": 1, "mode": 4 }] }],
            "buffers": [{ "byteLength": bin_chunk.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": 0, "byteLength": position_bytes.len(), "target": 34962 },
                { "buffer": 0, "byteOffset": index_offset, "byteLength": index_bytes.len(), "target": 34963 }
            ],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": child_positions.len(), "type": "VEC3", "max": max, "min": min },
                { "bufferView": 1, "componentType": 5123, "count": indices.len(), "type": "SCALAR" }
            ]
        });

        build_glb_chunks(&json, &bin_chunk)
    }

    /// Serializes JSON and BIN data into a GLB v2 container.
    fn build_glb_chunks(json: &serde_json::Value, bin_chunk: &[u8]) -> Vec<u8> {
        let mut json_bytes = serde_json::to_vec(json).unwrap();
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }

        let total_len = 12 + 8 + json_bytes.len() + 8 + bin_chunk.len();
        let mut glb = Vec::with_capacity(total_len);
        glb.extend_from_slice(&0x4654_6C67_u32.to_le_bytes()); // "glTF"
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(u32::try_from(total_len).unwrap()).to_le_bytes());
        glb.extend_from_slice(&(u32::try_from(json_bytes.len()).unwrap()).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534A_u32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(u32::try_from(bin_chunk.len()).unwrap()).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942_u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(bin_chunk);
        glb
    }

    #[test]
    fn minimal_glb_roundtrips_positions_and_indices() {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = [0_u16, 1, 2];
        let glb = build_test_glb(&positions, &indices, [0.0, 0.0, 0.0]);

        let gltf = Gltf::from_slice(&glb).expect("valid test GLB");
        let primitive = first_primitive(&gltf);

        let transform = mat4_identity();
        let read_pos = read_positions(&gltf, &primitive, &transform).expect("positions readable");
        assert_eq!(read_pos, positions.to_vec());

        let read_idx = read_indices(&gltf, &primitive).expect("indices readable");
        assert_eq!(read_idx, vec![0, 1, 2]);
    }

    #[test]
    fn node_translation_applies_to_positions() {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let glb = build_test_glb(&positions, &[0, 1, 2], [10.0, 20.0, 30.0]);

        let gltf = Gltf::from_slice(&glb).unwrap();
        let primitive = first_primitive(&gltf);
        let node = gltf.default_scene().unwrap().nodes().next().unwrap();
        let transform = mat4_mul(&mat4_identity(), &node.transform().matrix());

        let read_pos = read_positions(&gltf, &primitive, &transform).unwrap();
        assert_f32_array_eq(&read_pos[0], &[10.0, 20.0, 30.0]);
        assert_f32_array_eq(&read_pos[1], &[11.0, 20.0, 30.0]);
        assert_f32_array_eq(&read_pos[2], &[10.0, 21.0, 30.0]);
    }

    #[test]
    fn missing_normals_fall_back_to_up() {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let glb = build_test_glb(&positions, &[0, 1, 2], [0.0; 3]);

        let gltf = Gltf::from_slice(&glb).unwrap();
        let primitive = first_primitive(&gltf);
        let normals = read_normals(&gltf, &primitive, &mat4_identity());

        assert_eq!(normals, vec![[0.0, 1.0, 0.0]; 3]);
    }

    #[test]
    fn material_params_read_pbr_and_unlit_extension() {
        let glb = build_test_glb(&[[0.0_f32; 3]; 3], &[0, 1, 2], [0.0; 3]);
        let gltf = Gltf::from_slice(&glb).unwrap();
        let material = first_primitive(&gltf).material();

        let params = material_params(&material);
        assert_f32_array_eq(&params.base_color_factor, &[1.0, 0.0, 0.0, 1.0]);
        assert!((params.metallic_factor - 0.1).abs() < f32::EPSILON);
        assert!((params.roughness_factor - 0.2).abs() < f32::EPSILON);
        assert!(params.double_sided);
        assert!(params.unlit);
    }

    #[test]
    fn compute_bounding_box_matches_positions() {
        let positions = [[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let glb = build_test_glb(&positions, &[0, 1, 2], [0.0; 3]);
        let gltf = Gltf::from_slice(&glb).unwrap();

        let (min, max) = compute_bounding_box(&gltf);
        assert_f32_array_eq(&min, &[0.0, 0.0, 0.0]);
        assert_f32_array_eq(&max, &[2.0, 3.0, 0.0]);
    }

    #[test]
    fn triangle_mode_helpers_consistent() {
        assert!(is_triangle_mode(gltf::mesh::Mode::Triangles));
        assert!(!is_triangle_mode(gltf::mesh::Mode::Points));
        assert_eq!(triangle_count(gltf::mesh::Mode::Triangles, 6, 4), 2);
        assert_eq!(triangle_count(gltf::mesh::Mode::TriangleStrip, 5, 0), 3);
    }

    #[test]
    fn non_indexed_primitive_returns_no_indices() {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let glb = build_non_indexed_glb(&positions, [0.0; 3]);

        let gltf = Gltf::from_slice(&glb).unwrap();
        let primitive = first_primitive(&gltf);

        let read_pos = read_positions(&gltf, &primitive, &mat4_identity()).unwrap();
        assert_eq!(read_pos, positions.to_vec());
        assert!(read_indices(&gltf, &primitive).is_none());
    }

    #[test]
    fn supplied_normals_are_transformed() {
        let positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = [[0.0_f32, 0.0, 1.0]; 3];
        let glb = build_indexed_glb_with_normals(&positions, &[0, 1, 2], &normals, [0.0; 3]);

        let gltf = Gltf::from_slice(&glb).unwrap();
        let primitive = first_primitive(&gltf);
        let read = read_normals(&gltf, &primitive, &mat4_identity());

        assert_eq!(read, normals.to_vec());
    }

    #[test]
    fn hierarchy_accumulates_parent_transforms() {
        let child_positions = [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let glb = build_hierarchy_glb(
            &child_positions,
            &[0, 1, 2],
            [5.0, 0.0, 0.0],
            [0.0, 7.0, 0.0],
        );

        let gltf = Gltf::from_slice(&glb).unwrap();
        let (min, max) = compute_bounding_box(&gltf);

        assert_f32_array_eq(&min, &[5.0, 7.0, 0.0]);
        assert_f32_array_eq(&max, &[6.0, 8.0, 0.0]);
    }

    #[test]
    fn gltf_parses_minimal_glb_and_extraction_matches_input() {
        let positions = [[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let glb = build_test_glb(&positions, &[0_u16, 1, 2], [1.0, 0.0, 0.0]);
        let gltf = gltf::Gltf::from_slice(&glb).expect("valid test GLB");

        let (min, max) = compute_bounding_box(&gltf);
        assert_f32_array_eq(&min, &[1.0, 0.0, 0.0]);
        assert_f32_array_eq(&max, &[3.0, 3.0, 0.0]);

        let primitive = first_primitive(&gltf);
        let params = material_params(&primitive.material());
        assert_f32_array_eq(&params.base_color_factor, &[1.0, 0.0, 0.0, 1.0]);
        assert!((params.metallic_factor - 0.1).abs() < f32::EPSILON);
        assert!((params.roughness_factor - 0.2).abs() < f32::EPSILON);
        assert!(params.double_sided);
        assert!(params.unlit);
    }

    fn first_primitive(gltf: &Gltf) -> gltf::Primitive<'_> {
        gltf.default_scene()
            .expect("default scene")
            .nodes()
            .next()
            .expect("root node")
            .mesh()
            .expect("mesh")
            .primitives()
            .next()
            .expect("primitive")
    }
}
