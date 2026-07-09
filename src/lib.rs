//! Cadiotheka — the open hub for CAD creators.
//!
//! This crate provides the [`CadiothekaApp`] hub UI. It can run natively via
//! [`eframe::run_native`] or in a browser via [`eframe::WebRunner`].

mod app;
pub mod pages {
    pub mod hub;
    pub mod login;
}
pub mod i18n;

pub use app::CadiothekaApp;
