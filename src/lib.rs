//! Cadiotheka — the open hub for CAD creators.
//!
//! This crate provides the [`CadiothekaApp`] hub UI. It can run natively via
//! [`eframe::run_native`] or in a browser via [`eframe::WebRunner`].

mod app;
pub mod components {
    pub mod card;
    pub mod dotted_background;
    pub mod footer;
    pub mod grid;
    pub mod header;
    pub mod search_bar;
    pub mod sort_bar;

    pub use card::{Card, CardData, IconUrl};
    pub use dotted_background::DottedBackground;
    pub use footer::Footer;
    pub use grid::Grid;
    pub use header::{Header, View};
    pub use search_bar::SearchBar;
    pub use sort_bar::{SortBar, SortBy, SortOrder, SortSelection};
}
pub mod i18n;
pub mod pages {
    pub mod hub;

    pub use hub::Hub;
}
pub mod platforms;
pub mod tags;
pub mod utils;

pub use app::CadiothekaApp;
