//! Cadiotheka — the open hub for CAD creators.
//!
//! This crate provides the [`CadiothekaApp`] hub UI. It can run natively via
//! [`eframe::run_native`] or in a browser via [`eframe::WebRunner`].

mod app;
pub mod components {
    pub(crate) mod builders {
        pub mod dotted_background;
        pub mod keycap;

        pub use dotted_background::DottedBackground;
        pub use keycap::Keycap;
    }
    pub(crate) mod card;
    pub(crate) mod footer;
    pub(crate) mod grid;
    pub(crate) mod header;
    pub(crate) mod project_popup;
    pub(crate) mod search_bar;

    pub use builders::{DottedBackground, Keycap};
    pub use card::{Card, CardAction, CardData, IconUrl};
    pub use footer::Footer;
    pub use grid::Grid;
    pub use header::{Header, View};
    pub use project_popup::ProjectPopup;
    pub use search_bar::SearchBar;
}
pub mod engines {
    pub(crate) mod filter;
    pub(crate) mod query;
    pub(crate) mod suggestions;

    pub use filter::SearchEngine;
    pub use query::{ParsedQuery, SortBy, SortOrder, SortSelection};
    pub use suggestions::{Suggestion, SuggestionKind};
}
pub(crate) mod fixture;
pub mod i18n;
pub mod pages {
    pub(crate) mod hub;

    pub use hub::Hub;
}
pub mod platforms;
pub mod tags;
pub(crate) mod utils;

pub use app::CadiothekaApp;
