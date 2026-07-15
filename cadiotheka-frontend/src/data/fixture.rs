use serde::{Deserialize, Serialize};

use super::card::CardData;

/// Top-level fixture container for content cards.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CardsFixture {
    /// All cards in the fixture.
    pub cards: Vec<CardData>,
}
