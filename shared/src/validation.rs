//! Shared validation constants used by the frontend and backend.
//!
//! These limits are enforced by the backend and can be used by the frontend
//! for early client-side validation.

/// Maximum allowed length for a project title.
pub const MAX_TITLE_LENGTH: usize = 100;
/// Maximum allowed length for a project description.
pub const MAX_DESCRIPTION_LENGTH: usize = 5000;
/// Maximum allowed size for an uploaded project IFC model, in bytes.
pub const MAX_IFC_SIZE_BYTES: usize = 25 * 1024 * 1024; // 25 MiB
