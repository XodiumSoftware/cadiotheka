//! Per-primitive metadata for the 3D viewer.
//!
//! The backend emits a sidecar JSON file alongside the GLB that maps each
//! primitive index to the IFC express id (and any available name) of the object
//! it represents. The viewer uses this to turn a raycast hit into a meaningful
//! object selection.

use serde::Deserialize;

/// Metadata attached to a single uploaded primitive.
#[derive(Clone, Debug, Deserialize)]
pub struct PrimitiveMetadata {
    /// IFC express id of the product that produced this primitive, if known.
    pub express_id: Option<u32>,
    /// Human-readable name of the IFC product, if present in the GLB node.
    pub name: Option<String>,
}

/// Full sidecar metadata file returned by `/data/projects/:id/glb-metadata`.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct MetadataFile {
    primitives: Vec<PrimitiveMetadata>,
}

/// The result of a successful click on a model object.
#[derive(Clone, Debug)]
pub struct ObjectHit {
    /// Index of the intersected primitive in the renderer's model list.
    pub primitive_index: usize,
    /// World-space position of the intersection point.
    pub position: [f32; 3],
    /// IFC express id of the clicked object, when metadata is available.
    pub express_id: Option<u32>,
    /// Human-readable name of the clicked object, when metadata is available.
    pub name: Option<String>,
}

/// Fetches primitive metadata for the given URL.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_metadata(url: &str) -> Option<Vec<PrimitiveMetadata>> {
    use gloo_net::http::Request;

    let response = Request::get(url).send().await.ok()?;
    if !response.ok() {
        return None;
    }
    let text = response.text().await.ok()?;
    let file: MetadataFile = serde_json::from_str(&text).ok()?;
    Some(file.primitives)
}

/// Non-WASM stub: metadata fetching is not supported outside the browser.
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_metadata(_url: &str) -> Option<Vec<PrimitiveMetadata>> {
    None
}
