//! Project data and API helpers.
//!
//! This module re-exports the public types and functions from
//! [`project_types`](crate::data::project_types) and
//! [`project_api`](crate::data::project_api) so callers can continue to use
//! `crate::data::project::*`.

pub use super::project_api::*;
pub use super::project_types::{
    IconUrl, ProjectCreationResult, ProjectData, ProjectPatch, icon_src_from_key, ifc_src_from_key,
    new_project_payload,
};
