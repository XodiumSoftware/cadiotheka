//! Serde adapters for columns that hold JSON text.
//!
//! D1 stores arrays (`tags`, `favorites`, `collaborator_ids`, ...) as TEXT
//! containing a JSON array. The frontend and backend both read and write that
//! same wire format, so the adapters live here instead of being duplicated in
//! each crate.

use serde::{Deserialize, Deserializer, Serializer};

/// Serializes a string list as a JSON array encoded into a single TEXT value.
///
/// # Errors
///
/// Returns the serializer's error when the list cannot be encoded as JSON or
/// the resulting string cannot be written.
pub fn serialize<T, S>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    T: serde::Serialize,
    S: Serializer,
{
    serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
}

/// Deserializes a TEXT value holding a JSON array of strings.
///
/// # Errors
///
/// Returns a deserialization error when the value is not a string or the
/// string is not a valid JSON array of strings.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    serde_json::from_str(&text).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Row {
        #[serde(with = "super")]
        items: Vec<String>,
    }

    #[test]
    fn roundtrips_through_a_json_string_column() -> Result<(), serde_json::Error> {
        let row = Row {
            items: vec!["3d_model".to_owned(), "vehicle".to_owned()],
        };
        let json = serde_json::to_string(&row)?;
        assert_eq!(json, r#"{"items":"[\"3d_model\",\"vehicle\"]"}"#);
        let decoded: Row = serde_json::from_str(&json)?;
        assert_eq!(decoded, row);
        Ok(())
    }

    #[test]
    fn roundtrips_an_empty_list() -> Result<(), serde_json::Error> {
        let row = Row { items: vec![] };
        let json = serde_json::to_string(&row)?;
        assert_eq!(json, r#"{"items":"[]"}"#);
        let decoded: Row = serde_json::from_str(&json)?;
        assert_eq!(decoded, row);
        Ok(())
    }

    #[test]
    fn rejects_values_that_are_not_json_arrays() {
        let decoded = serde_json::from_str::<Row>(r#"{"items":"{not json}"}"#);
        assert!(decoded.is_err());
    }
}
