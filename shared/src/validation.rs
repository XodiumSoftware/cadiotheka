//! Shared validation rules used by the frontend and backend.
//!
//! The backend is authoritative, but the frontend runs the same checks for
//! early feedback, so the limits, the rejection messages, and the counting
//! rule live here to keep both sides in agreement.
//!
//! Length limits are measured in **characters** (Unicode scalar values), not
//! bytes: the limits are meant to bound what a user types, and a byte-based
//! check would reject an otherwise valid title early on any non-ASCII text.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tags::Tag;

/// Maximum allowed length for a project title, in characters.
pub const MAX_TITLE_LENGTH: usize = 100;
/// Maximum allowed length for a project description, in characters.
pub const MAX_DESCRIPTION_LENGTH: usize = 5000;
/// Maximum allowed length for a passkey's user-assigned display name.
pub const MAX_PASSKEY_NAME_LENGTH: usize = 64;
/// Maximum length for a user-written bio, in characters.
///
/// Matches GitHub's profile bio limit, which users compare against.
pub const MAX_BIO_LENGTH: usize = 160;
/// Maximum number of credited collaborators a project may list.
pub const MAX_COLLABORATORS: usize = 50;
/// Maximum allowed size for an uploaded project IFC model, in bytes.
pub const MAX_IFC_SIZE_BYTES: usize = 25 * 1024 * 1024; // 25 MiB
/// Maximum size for the stored `viewer_preferences` JSON blob, in bytes.
///
/// This is machine-written JSON rather than user-facing text, so it is bounded
/// by encoded size instead of character count.
pub const MAX_VIEWER_PREFERENCES_BYTES: usize = 4_096;

/// Returns the number of characters in `text`.
///
/// This is the unit every length limit in this module is measured in, so
/// client-side counters, `<input maxlength>` attributes, and server-side
/// checks all agree.
#[must_use]
pub fn char_count(text: &str) -> usize {
    text.chars().count()
}

/// Returns `true` when `text` exceeds `limit` characters.
#[must_use]
pub fn exceeds(text: &str, limit: usize) -> bool {
    char_count(text) > limit
}

/// Returns the number of UTF-16 code units in `text`.
///
/// Browsers count `<input maxlength>` and report `input.value.length` in UTF-16
/// code units, so character counters that sit next to a `maxlength` input must
/// use this unit to agree with the browser. It differs from [`char_count`]
/// only for astral characters, such as emoji, which take two code units.
#[must_use]
pub fn input_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Message returned when a project title exceeds [`MAX_TITLE_LENGTH`].
#[must_use]
pub fn title_too_long_message() -> String {
    format!("Title must be {MAX_TITLE_LENGTH} characters or fewer")
}

/// Message returned when a project description exceeds [`MAX_DESCRIPTION_LENGTH`].
#[must_use]
pub fn description_too_long_message() -> String {
    format!("Description must be {MAX_DESCRIPTION_LENGTH} characters or fewer")
}

/// Message returned when a project lists more than [`MAX_COLLABORATORS`].
#[must_use]
pub fn too_many_collaborators_message() -> String {
    format!("A project can have at most {MAX_COLLABORATORS} collaborators")
}

/// Message returned when a project carries a tag id the server does not know.
#[must_use]
pub fn unknown_tag_message() -> &'static str {
    "One or more selected tags are no longer available"
}

/// Message returned when a bio exceeds [`MAX_BIO_LENGTH`].
#[must_use]
pub fn bio_too_long_message() -> String {
    format!("Bio must be {MAX_BIO_LENGTH} characters or fewer")
}

/// Message returned when viewer preferences exceed
/// [`MAX_VIEWER_PREFERENCES_BYTES`].
#[must_use]
pub fn viewer_preferences_too_large_message() -> String {
    format!("Viewer preferences must be {MAX_VIEWER_PREFERENCES_BYTES} bytes or fewer")
}

/// The client-editable fields of a project, as far as validation is concerned.
///
/// Both the create and patch endpoints validate through this shape, so a field
/// added to one payload cannot silently skip the shared rules.
#[derive(Debug, Clone, Copy)]
pub struct ProjectFields<'a> {
    /// Project title.
    pub title: Option<&'a str>,
    /// Project description (Markdown).
    pub description: Option<&'a str>,
    /// Account ids of credited collaborators.
    pub collaborator_ids: Option<&'a [String]>,
    /// Wire ids of the tags categorizing the project.
    pub tags: Option<&'a [String]>,
}

impl<'a> ProjectFields<'a> {
    /// Returns the fields present in a create payload.
    #[must_use]
    pub const fn new(
        title: &'a str,
        description: &'a str,
        collaborator_ids: &'a [String],
        tags: &'a [String],
    ) -> Self {
        Self {
            title: Some(title),
            description: Some(description),
            collaborator_ids: Some(collaborator_ids),
            tags: Some(tags),
        }
    }
}

/// Validates project fields, returning one message per invalid field.
///
/// Only fields that are present are checked, which lets the patch endpoint
/// validate just the values a request actually changes. An empty `errors` map
/// means every provided field is acceptable.
///
/// Field names in the result match the JSON keys the frontend submits, so a
/// client can map each message onto the input that produced it.
#[must_use]
pub fn validate_project_fields(fields: ProjectFields<'_>) -> ValidationErrorResponse {
    let mut errors = HashMap::new();
    if fields
        .title
        .is_some_and(|title| exceeds(title, MAX_TITLE_LENGTH))
    {
        errors.insert("title".to_owned(), title_too_long_message());
    }
    if fields
        .description
        .is_some_and(|description| exceeds(description, MAX_DESCRIPTION_LENGTH))
    {
        errors.insert("description".to_owned(), description_too_long_message());
    }
    if let Some(collaborator_ids) = fields.collaborator_ids
        && collaborator_ids.len() > MAX_COLLABORATORS
    {
        errors.insert(
            "collaborator_ids".to_owned(),
            too_many_collaborators_message(),
        );
    }
    if fields
        .tags
        .is_some_and(|tags| tags.iter().any(|id| Tag::from_id(id).is_none()))
    {
        errors.insert("tags".to_owned(), unknown_tag_message().to_owned());
    }
    ValidationErrorResponse::new(errors)
}

/// Body returned by the backend when field validation fails.
///
/// Mirrors the `{ "errors": { field: message } }` shape the frontend parses,
/// so both sides can share one definition of the contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    /// One message per invalid field, keyed by field name.
    pub errors: HashMap<String, String>,
}

impl ValidationErrorResponse {
    /// Wraps a map of field errors.
    #[must_use]
    pub const fn new(errors: HashMap<String, String>) -> Self {
        Self { errors }
    }

    /// Returns `true` when no field failed validation.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_report_the_actual_limits() {
        assert_eq!(
            title_too_long_message(),
            "Title must be 100 characters or fewer"
        );
        assert_eq!(
            description_too_long_message(),
            "Description must be 5000 characters or fewer"
        );
        assert_eq!(
            bio_too_long_message(),
            "Bio must be 160 characters or fewer"
        );
        assert_eq!(
            viewer_preferences_too_large_message(),
            "Viewer preferences must be 4096 bytes or fewer"
        );
    }

    /// The bio limit counts characters, so a non-ASCII bio of the same visual
    /// length is not rejected for its byte size.
    #[test]
    fn non_ascii_bios_are_measured_in_characters() {
        let bio = "é".repeat(MAX_BIO_LENGTH);
        assert!(bio.len() > MAX_BIO_LENGTH, "the byte length is larger");
        assert!(!exceeds(&bio, MAX_BIO_LENGTH));
        assert!(exceeds(&"é".repeat(MAX_BIO_LENGTH + 1), MAX_BIO_LENGTH));
    }

    #[test]
    fn char_count_counts_scalar_values_not_bytes() {
        assert_eq!(char_count("abc"), 3);
        assert_eq!(char_count("é"), 1);
        assert_eq!("é".len(), 2, "the byte length is larger");
        assert_eq!(char_count("模型"), 2);
        assert_eq!(char_count("👷‍♀️"), 4);
    }

    #[test]
    fn input_len_counts_utf16_code_units() {
        assert_eq!(input_len("abc"), 3);
        assert_eq!(input_len("é"), 1);
        assert_eq!(input_len("模型"), 2);
        assert_eq!(input_len("👷"), 2, "astral characters take two code units");
        assert_eq!(char_count("👷"), 1);
    }

    #[test]
    fn non_ascii_titles_within_the_limit_are_accepted() {
        let title = "ü".repeat(MAX_TITLE_LENGTH);
        assert!(!exceeds(&title, MAX_TITLE_LENGTH));
        assert!(title.len() > MAX_TITLE_LENGTH, "the byte length is larger");
    }

    #[test]
    fn a_title_over_the_character_limit_is_rejected() {
        let title = "ü".repeat(MAX_TITLE_LENGTH + 1);
        let errors = validate_project_fields(ProjectFields::new(&title, "", &[], &[]));
        assert_eq!(errors.errors.get("title"), Some(&title_too_long_message()));
        assert!(!errors.errors.contains_key("description"));
    }

    #[test]
    fn valid_fields_produce_no_errors() {
        let errors = validate_project_fields(ProjectFields::new("Bike", "Nice model", &[], &[]));
        assert!(errors.is_empty());
    }

    #[test]
    fn absent_fields_are_not_validated() {
        let errors = validate_project_fields(ProjectFields {
            title: None,
            description: None,
            collaborator_ids: None,
            tags: None,
        });
        assert!(errors.is_empty());
    }

    #[test]
    fn long_descriptions_are_rejected() {
        let description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let errors = validate_project_fields(ProjectFields::new("Bike", &description, &[], &[]));
        assert_eq!(
            errors.errors.get("description"),
            Some(&description_too_long_message())
        );
        assert!(!errors.errors.contains_key("title"));
    }

    #[test]
    fn too_many_collaborators_are_rejected() {
        let collaborators = vec!["acc".to_owned(); MAX_COLLABORATORS + 1];
        let errors =
            validate_project_fields(ProjectFields::new("Bike", "Model", &collaborators, &[]));
        assert_eq!(
            errors.errors.get("collaborator_ids"),
            Some(&too_many_collaborators_message())
        );
    }

    #[test]
    fn the_collaborator_limit_itself_is_accepted() {
        let collaborators = vec!["acc".to_owned(); MAX_COLLABORATORS];
        let errors =
            validate_project_fields(ProjectFields::new("Bike", "Model", &collaborators, &[]));
        assert!(errors.is_empty());
    }

    #[test]
    fn unknown_tag_ids_are_rejected() {
        let tags = vec!["vehicle".to_owned(), "retired_tag".to_owned()];
        let errors = validate_project_fields(ProjectFields::new("Bike", "Model", &[], &tags));
        assert_eq!(
            errors.errors.get("tags"),
            Some(&unknown_tag_message().to_owned())
        );
    }

    #[test]
    fn known_tag_ids_are_accepted() {
        let tags = vec!["vehicle".to_owned(), "3d_model".to_owned()];
        let errors = validate_project_fields(ProjectFields::new("Bike", "Model", &[], &tags));
        assert!(errors.is_empty());
    }

    #[test]
    fn error_response_roundtrips_through_json() -> Result<(), serde_json::Error> {
        let mut errors = HashMap::new();
        errors.insert("title".to_owned(), title_too_long_message());
        let response = ValidationErrorResponse::new(errors);
        let json = serde_json::to_string(&response)?;
        assert!(json.starts_with(r#"{"errors":{"title":"#));
        let decoded: ValidationErrorResponse = serde_json::from_str(&json)?;
        assert_eq!(decoded, response);
        Ok(())
    }
}
