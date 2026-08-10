//! Re-export of shared content tags for Cadiotheka.
//!
//! Tags are no longer defined in the frontend; they live in the `shared`
//! workspace crate so both the frontend and backend use the same wire ids,
//! labels, and colors. Project rows still store tag wire ids as JSON arrays,
//! so the frontend resolves labels and colors through the re-exported helpers
//! below.

pub use shared::tags::{Tag, tag_color, tag_label};
