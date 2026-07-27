//! Minimal GLB/glTF parser for rendering IFC models produced by `ifc-lite-wasm`.

#![allow(
    clippy::pedantic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::collapsible_if,
    clippy::get_first,
    clippy::redundant_locals
)]

use std::collections::HashMap;

/// Parsed GLB document with a single embedded binary buffer.
#[derive(Debug, Clone)]
pub struct GltfDocument {
    /// Scene nodes.
    pub nodes: Vec<GltfNode>,
    /// Meshes referenced by nodes.
    pub meshes: Vec<GltfMesh>,
    /// Materials referenced by primitives.
    pub materials: Vec<GltfMaterial>,
    /// Accessors into the binary buffer.
    pub accessors: Vec<GltfAccessor>,
    /// Buffer views describing accessor layouts.
    pub buffer_views: Vec<GltfBufferView>,
    /// The single embedded binary chunk.
    pub buffer: Vec<u8>,
    /// Default scene index.
    pub scene: Option<usize>,
}

/// A scene node with an optional mesh and local transform.
#[derive(Debug, Clone)]
pub struct GltfNode {
    /// Index into [`GltfDocument::meshes`], if any.
    pub mesh: Option<usize>,
    /// Child node indices.
    pub children: Vec<usize>,
    /// Translation component.
    pub translation: [f32; 3],
    /// Rotation quaternion `[x, y, z, w]`.
    pub rotation: [f32; 4],
    /// Scale component.
    pub scale: [f32; 3],
    /// Optional 4×4 column-major transform matrix.
    pub matrix: Option<[f32; 16]>,
}

/// A mesh composed of primitives.
#[derive(Debug, Clone)]
pub struct GltfMesh {
    /// Primitives (typically triangles).
    pub primitives: Vec<GltfPrimitive>,
}

/// A single drawable primitive.
#[derive(Debug, Clone)]
pub struct GltfPrimitive {
    /// Accessor indices keyed by attribute semantic, e.g. `"POSITION"`.
    pub attributes: HashMap<String, usize>,
    /// Accessor index for element indices, if indexed.
    pub indices: Option<usize>,
    /// Material index, if any.
    pub material: Option<usize>,
    /// Render mode (default `4` = triangles).
    pub mode: u32,
}

/// Accessor describing a view into a buffer.
#[derive(Debug, Clone)]
pub struct GltfAccessor {
    /// Index into [`GltfDocument::buffer_views`].
    pub buffer_view: usize,
    /// Byte offset into the buffer view.
    pub byte_offset: usize,
    /// GL component type constant.
    pub component_type: u32,
    /// Number of elements.
    pub count: usize,
    /// Accessor type name, e.g. `"VEC3"`.
    pub type_name: String,
    /// Minimum component values, if provided.
    pub min: Option<Vec<f32>>,
    /// Maximum component values, if provided.
    pub max: Option<Vec<f32>>,
}

/// A view into a buffer.
#[derive(Debug, Clone)]
pub struct GltfBufferView {
    /// Buffer index (always `0` for embedded GLB).
    pub buffer: usize,
    /// Byte offset into the buffer.
    pub byte_offset: usize,
    /// Byte length of the view.
    pub byte_length: usize,
    /// Byte stride between elements, if applicable.
    pub byte_stride: Option<usize>,
    /// WebGL buffer target hint, if provided.
    pub target: Option<u32>,
}

/// Material with PBR parameters.
#[derive(Debug, Clone, Copy)]
pub struct GltfMaterial {
    /// Base color RGBA factor.
    pub base_color_factor: [f32; 4],
}

impl Default for GltfMaterial {
    fn default() -> Self {
        Self {
            base_color_factor: [0.7, 0.7, 0.7, 1.0],
        }
    }
}

/// Errors that can occur while parsing a GLB document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GltfError {
    /// The magic header is not a glTF binary file.
    InvalidMagic,
    /// Only glTF binary version 2 is supported.
    UnsupportedVersion,
    /// The file ended unexpectedly.
    Truncated,
    /// The JSON chunk could not be decoded or parsed.
    InvalidJson(String),
    /// A required chunk is missing.
    MissingChunk(&'static str),
    /// An accessor index or layout is invalid.
    InvalidAccessor(&'static str),
}

impl std::fmt::Display for GltfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "not a valid GLB file"),
            Self::UnsupportedVersion => write!(f, "only GLB version 2 is supported"),
            Self::Truncated => write!(f, "GLB file is truncated"),
            Self::InvalidJson(msg) => write!(f, "invalid glTF JSON: {msg}"),
            Self::MissingChunk(name) => write!(f, "missing {name} chunk"),
            Self::InvalidAccessor(msg) => write!(f, "invalid accessor: {msg}"),
        }
    }
}

impl std::error::Error for GltfError {}

/// Parses a GLB byte buffer into a [`GltfDocument`].
pub fn parse_glb(bytes: &[u8]) -> Result<GltfDocument, GltfError> {
    if bytes.len() < 12 {
        return Err(GltfError::Truncated);
    }

    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if magic != 0x4654_6C67 {
        return Err(GltfError::InvalidMagic);
    }

    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != 2 {
        return Err(GltfError::UnsupportedVersion);
    }

    let length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    if bytes.len() < length {
        return Err(GltfError::Truncated);
    }

    let mut offset = 12;
    let mut json_bytes: Option<&[u8]> = None;
    let mut bin_bytes: Option<&[u8]> = None;

    while offset < length {
        if offset + 8 > length {
            return Err(GltfError::Truncated);
        }
        let chunk_length = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        let chunk_type = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;
        if offset + chunk_length > length {
            return Err(GltfError::Truncated);
        }
        let chunk_data = &bytes[offset..offset + chunk_length];
        match chunk_type {
            0x4E4F_534A => json_bytes = Some(chunk_data),
            0x004E_4942 => bin_bytes = Some(chunk_data),
            _ => {}
        }
        offset += chunk_length;
    }

    let json_bytes = json_bytes.ok_or(GltfError::MissingChunk("JSON"))?;
    let bin_bytes = bin_bytes.ok_or(GltfError::MissingChunk("BIN"))?;

    let json: serde_json::Value =
        serde_json::from_slice(json_bytes).map_err(|e| GltfError::InvalidJson(e.to_string()))?;

    parse_gltf(json, bin_bytes.to_vec())
}

fn parse_gltf(json: serde_json::Value, buffer: Vec<u8>) -> Result<GltfDocument, GltfError> {
    let accessors = json
        .get("accessors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(parse_accessor)
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(Vec::new()))?;

    let buffer_views = json
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(parse_buffer_view)
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(Vec::new()))?;

    let materials = json
        .get("materials")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(parse_material)
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(Vec::new()))?;

    let meshes = json
        .get("meshes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|m| parse_mesh(m, &materials))
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(Vec::new()))?;

    let nodes = json
        .get("nodes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(parse_node).collect::<Result<Vec<_>, _>>())
        .unwrap_or(Ok(Vec::new()))?;

    let scene = json
        .get("scene")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    Ok(GltfDocument {
        nodes,
        meshes,
        materials,
        accessors,
        buffer_views,
        buffer,
        scene,
    })
}

fn parse_accessor(value: &serde_json::Value) -> Result<GltfAccessor, GltfError> {
    let obj = value
        .as_object()
        .ok_or(GltfError::InvalidAccessor("not an object"))?;
    Ok(GltfAccessor {
        buffer_view: obj
            .get("bufferView")
            .and_then(|v| v.as_u64())
            .ok_or(GltfError::InvalidAccessor("missing bufferView"))? as usize,
        byte_offset: obj.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        component_type: obj
            .get("componentType")
            .and_then(|v| v.as_u64())
            .ok_or(GltfError::InvalidAccessor("missing componentType"))?
            as u32,
        count: obj
            .get("count")
            .and_then(|v| v.as_u64())
            .ok_or(GltfError::InvalidAccessor("missing count"))? as usize,
        type_name: obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or(GltfError::InvalidAccessor("missing type"))?
            .to_string(),
        min: obj.get("min").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_f64().map(|f| f as f32))
                .collect()
        }),
        max: obj.get("max").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_f64().map(|f| f as f32))
                .collect()
        }),
    })
}

fn parse_buffer_view(value: &serde_json::Value) -> Result<GltfBufferView, GltfError> {
    let obj = value
        .as_object()
        .ok_or(GltfError::InvalidAccessor("bufferView not an object"))?;
    Ok(GltfBufferView {
        buffer: obj.get("buffer").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        byte_offset: obj.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        byte_length: obj
            .get("byteLength")
            .and_then(|v| v.as_u64())
            .ok_or(GltfError::InvalidAccessor("missing byteLength"))? as usize,
        byte_stride: obj
            .get("byteStride")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize),
        target: obj.get("target").and_then(|v| v.as_u64()).map(|v| v as u32),
    })
}

fn parse_material(value: &serde_json::Value) -> Result<GltfMaterial, GltfError> {
    let obj = value.as_object().unwrap_or(&serde_json::Map::new()).clone();
    let mut material = GltfMaterial::default();
    if let Some(pbr) = obj.get("pbrMetallicRoughness") {
        if let Some(factor) = pbr.get("baseColorFactor") {
            if let Some(arr) = factor.as_array() {
                for (i, v) in arr.iter().take(4).enumerate() {
                    material.base_color_factor[i] = v.as_f64().map(|f| f as f32).unwrap_or(1.0);
                }
            }
        }
    }
    Ok(material)
}

fn parse_mesh(
    value: &serde_json::Value,
    materials: &[GltfMaterial],
) -> Result<GltfMesh, GltfError> {
    let obj = value
        .as_object()
        .ok_or(GltfError::InvalidAccessor("mesh not an object"))?;
    let primitives = obj
        .get("primitives")
        .and_then(|v| v.as_array())
        .ok_or(GltfError::InvalidAccessor("missing primitives"))?
        .iter()
        .map(|p| parse_primitive(p, materials))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GltfMesh { primitives })
}

fn parse_primitive(
    value: &serde_json::Value,
    _materials: &[GltfMaterial],
) -> Result<GltfPrimitive, GltfError> {
    let obj = value
        .as_object()
        .ok_or(GltfError::InvalidAccessor("primitive not an object"))?;
    let attributes = obj
        .get("attributes")
        .and_then(|v| v.as_object())
        .ok_or(GltfError::InvalidAccessor("missing attributes"))?
        .iter()
        .map(|(k, v)| {
            v.as_u64()
                .map(|idx| (k.clone(), idx as usize))
                .ok_or(GltfError::InvalidAccessor("invalid attribute index"))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(GltfPrimitive {
        attributes,
        indices: obj
            .get("indices")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize),
        material: obj
            .get("material")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize),
        mode: obj
            .get("mode")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(4),
    })
}

fn parse_node(value: &serde_json::Value) -> Result<GltfNode, GltfError> {
    let obj = value
        .as_object()
        .ok_or(GltfError::InvalidAccessor("node not an object"))?;
    let translation = obj
        .get("translation")
        .and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
                arr.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
                arr.get(2).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            ]
        })
        .unwrap_or([0.0; 3]);

    let rotation = obj
        .get("rotation")
        .and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
                arr.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
                arr.get(2).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
                arr.get(3).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32,
            ]
        })
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);

    let scale = obj
        .get("scale")
        .and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.get(0).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32,
                arr.get(1).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32,
                arr.get(2).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32,
            ]
        })
        .unwrap_or([1.0; 3]);

    let matrix = obj.get("matrix").and_then(|v| v.as_array()).map(|arr| {
        let mut m = [0.0_f32; 16];
        for (i, v) in arr.iter().take(16).enumerate() {
            m[i] = v.as_f64().map(|f| f as f32).unwrap_or(0.0);
        }
        m
    });

    let children = obj
        .get("children")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|n| n as usize))
                .collect()
        })
        .unwrap_or_default();

    Ok(GltfNode {
        mesh: obj.get("mesh").and_then(|v| v.as_u64()).map(|v| v as usize),
        children,
        translation,
        rotation,
        scale,
        matrix,
    })
}

/// Number of scalar components for a glTF accessor type.
pub fn accessor_type_components(type_name: &str) -> Result<usize, GltfError> {
    match type_name {
        "SCALAR" => Ok(1),
        "VEC2" => Ok(2),
        "VEC3" => Ok(3),
        "VEC4" => Ok(4),
        "MAT2" => Ok(4),
        "MAT3" => Ok(9),
        "MAT4" => Ok(16),
        _ => Err(GltfError::InvalidAccessor("unknown accessor type")),
    }
}

/// Size in bytes of a glTF component type.
pub fn component_type_size(component_type: u32) -> Result<usize, GltfError> {
    match component_type {
        5120 | 5121 => Ok(1),
        5122 | 5123 => Ok(2),
        5125 | 5126 => Ok(4),
        _ => Err(GltfError::InvalidAccessor("unknown component type")),
    }
}

/// Reads accessor data as `Vec<[f32; 3]>` for `VEC3` float accessors.
pub fn read_vec3_accessor(
    doc: &GltfDocument,
    accessor_index: usize,
) -> Result<Vec<[f32; 3]>, GltfError> {
    let accessor = doc
        .accessors
        .get(accessor_index)
        .ok_or(GltfError::InvalidAccessor("accessor index out of bounds"))?;
    if accessor.type_name != "VEC3" {
        return Err(GltfError::InvalidAccessor("expected VEC3 accessor"));
    }
    if accessor.component_type != 5126 {
        return Err(GltfError::InvalidAccessor("expected float component type"));
    }
    read_f32_vec3(doc, accessor)
}

/// Reads an index accessor as `Vec<u32>`.
pub fn read_index_accessor(
    doc: &GltfDocument,
    accessor_index: usize,
) -> Result<Vec<u32>, GltfError> {
    let accessor = doc
        .accessors
        .get(accessor_index)
        .ok_or(GltfError::InvalidAccessor("index accessor out of bounds"))?;
    let view = doc
        .buffer_views
        .get(accessor.buffer_view)
        .ok_or(GltfError::InvalidAccessor("index bufferView out of bounds"))?;
    let byte_offset = view.byte_offset + accessor.byte_offset;
    let count = accessor.count;

    let bytes = doc
        .buffer
        .get(byte_offset..byte_offset + count * component_type_size(accessor.component_type)?)
        .ok_or(GltfError::Truncated)?;

    match accessor.component_type {
        5123 => Ok(bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]) as u32)
            .take(count)
            .collect()),
        5125 => Ok(bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .take(count)
            .collect()),
        _ => Err(GltfError::InvalidAccessor(
            "unsupported index component type",
        )),
    }
}

fn read_f32_vec3(doc: &GltfDocument, accessor: &GltfAccessor) -> Result<Vec<[f32; 3]>, GltfError> {
    let view = doc
        .buffer_views
        .get(accessor.buffer_view)
        .ok_or(GltfError::InvalidAccessor("bufferView out of bounds"))?;
    let byte_offset = view.byte_offset + accessor.byte_offset;
    let stride = view.byte_stride.unwrap_or(3 * std::mem::size_of::<f32>());

    let mut result = Vec::with_capacity(accessor.count);
    for i in 0..accessor.count {
        let start = byte_offset + i * stride;
        let chunk = doc
            .buffer
            .get(start..start + 3 * std::mem::size_of::<f32>())
            .ok_or(GltfError::Truncated)?;
        result.push([
            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
            f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
            f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]),
        ]);
    }
    Ok(result)
}

/// Computes a world-space axis-aligned bounding box for the whole document.
pub fn compute_bounding_box(doc: &GltfDocument) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    let identity = mat4_identity();
    for node_index in doc.nodes.iter().enumerate().map(|(i, _)| i) {
        visit_node_bounds(doc, node_index, &identity, &mut min, &mut max);
    }

    if min[0].is_infinite() {
        min = [0.0; 3];
        max = [1.0; 3];
    }
    (min, max)
}

fn visit_node_bounds(
    doc: &GltfDocument,
    node_index: usize,
    parent_transform: &[[f32; 4]; 4],
    min: &mut [f32; 3],
    max: &mut [f32; 3],
) {
    let Some(node) = doc.nodes.get(node_index) else {
        return;
    };
    let transform = mat4_mul(parent_transform, &node_transform(node));

    if let Some(mesh_index) = node.mesh {
        if let Some(mesh) = doc.meshes.get(mesh_index) {
            for primitive in &mesh.primitives {
                if let Some(&position_index) = primitive.attributes.get("POSITION") {
                    if let Ok(positions) = read_vec3_accessor(doc, position_index) {
                        for p in positions {
                            let world = mat4_mul_vec3(&transform, p);
                            for i in 0..3 {
                                min[i] = min[i].min(world[i]);
                                max[i] = max[i].max(world[i]);
                            }
                        }
                    }
                }
            }
        }
    }

    for &child in &node.children {
        visit_node_bounds(doc, child, &transform, min, max);
    }
}

/// Computes the local transform matrix of a glTF node.
pub fn node_transform(node: &GltfNode) -> [[f32; 4]; 4] {
    if let Some(matrix) = node.matrix {
        return [
            [matrix[0], matrix[1], matrix[2], matrix[3]],
            [matrix[4], matrix[5], matrix[6], matrix[7]],
            [matrix[8], matrix[9], matrix[10], matrix[11]],
            [matrix[12], matrix[13], matrix[14], matrix[15]],
        ];
    }
    let translation = mat4_translation(node.translation);
    let rotation = mat4_from_quaternion(node.rotation);
    let scale = mat4_scale(node.scale);
    mat4_mul(&mat4_mul(&translation, &rotation), &scale)
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

fn mat4_translation(t: [f32; 3]) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t[0], t[1], t[2], 1.0],
    ]
}

fn mat4_scale(s: [f32; 3]) -> [[f32; 4]; 4] {
    [
        [s[0], 0.0, 0.0, 0.0],
        [0.0, s[1], 0.0, 0.0],
        [0.0, 0.0, s[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[allow(clippy::many_single_char_names)]
fn mat4_from_quaternion(q: [f32; 4]) -> [[f32; 4]; 4] {
    let [x, y, z, w] = q;
    let xx = 2.0 * x * x;
    let yy = 2.0 * y * y;
    let zz = 2.0 * z * z;
    let xy = 2.0 * x * y;
    let xz = 2.0 * x * z;
    let yz = 2.0 * y * z;
    let wx = 2.0 * w * x;
    let wy = 2.0 * w * y;
    let wz = 2.0 * w * z;

    [
        [1.0 - yy - zz, xy + wz, xz - wy, 0.0],
        [xy - wz, 1.0 - xx - zz, yz + wx, 0.0],
        [xz + wy, yz - wx, 1.0 - xx - yy, 0.0],
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
    normalize_vec3(&mut f);
    let up = up;
    let mut s = cross_vec3(&f, &up);
    normalize_vec3(&mut s);
    let mut u = cross_vec3(&s, &f);
    normalize_vec3(&mut u);

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

fn normalize_vec3(v: &mut [f32; 3]) {
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
