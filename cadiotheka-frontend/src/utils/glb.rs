//! Minimal glTF/GLB helpers for rendering IFC models.
//!
//! This module is a thin wrapper around the `gltf` crate that exposes the
//! small API surface required by the custom WebGL renderer.

pub use gltf::Gltf;
pub use gltf::mesh::util::ReadIndices;
pub use gltf::scene::Transform;

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

/// Builds a 4×4 identity matrix.
pub fn mat4_identity() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Multiplies two 4×4 matrices.
pub fn mat4_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0_f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

/// Transforms a 3D vector by a 4×4 matrix (assuming `w = 1.0`).
#[allow(clippy::many_single_char_names)]
pub fn mat4_mul_vec3(m: &[[f32; 4]; 4], v: [f32; 3]) -> [f32; 3] {
    let x = m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0];
    let y = m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1];
    let z = m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2];
    [x, y, z]
}

/// Converts a column-major 4×4 matrix to a flat `[f32; 16]` row-major array.
pub fn mat4_to_array(m: &[[f32; 4]; 4]) -> [f32; 16] {
    let mut result = [0.0_f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            result[col * 4 + row] = m[row][col];
        }
    }
    result
}

/// Extracts the upper-left 3×3 normal matrix from a 4×4 transform.
pub fn mat4_to_normal_matrix_3x3(m: &[[f32; 4]; 4]) -> [f32; 9] {
    [
        m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1], m[0][2], m[1][2], m[2][2],
    ]
}

/// Computes a perspective projection matrix.
pub fn perspective_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov_y * 0.5).tan();
    let nf = 1.0 / (near - far);

    [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (far + near) * nf,
        -1.0,
        0.0,
        0.0,
        2.0 * far * near * nf,
        0.0,
    ]
}

/// Computes a look-at view matrix.
pub fn look_at_matrix(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let mut f = [center[0] - eye[0], center[1] - eye[1], center[2] - eye[2]];
    normalize_vec3_mut(&mut f);
    let mut s = cross_vec3(&f, &up);
    normalize_vec3_mut(&mut s);
    let mut u = cross_vec3(&s, &f);
    normalize_vec3_mut(&mut u);

    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [
            -dot_vec3(&s, &eye),
            -dot_vec3(&u, &eye),
            dot_vec3(&f, &eye),
            1.0,
        ],
    ]
}

fn normalize_vec3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
}

fn normalize_vec3_mut(v: &mut [f32; 3]) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

fn cross_vec3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot_vec3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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
