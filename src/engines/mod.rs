//! Search engine for filtering, sorting, and suggesting cards.

pub mod filter;
pub mod query;
pub mod suggestions;

pub use filter::SearchEngine;
pub use query::{ParsedQuery, SortBy, SortOrder, SortSelection, parse_query};
pub use suggestions::{Suggestion, SuggestionKind, from_cards};
