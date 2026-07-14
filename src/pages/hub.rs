//! Main hub UI component shown after a successful login.
//!
//! The card catalog is currently loaded from the embedded fixture managed by
//! [`crate::fixture`]. See that module for notes on the planned move to a
//! runtime data source.

use crate::components::{CardAction, CardData, DottedBackground, Grid, ProjectPopup, SearchBar};
use crate::engines::SearchEngine;
use crate::fixture::load_cards;

/// Current loading state of the hub's card catalog.
#[derive(Debug, Default)]
enum LoadState {
    /// Catalog has not been loaded yet.
    #[default]
    Loading,
    /// Catalog is loaded and ready.
    Ready,
    /// Catalog failed to load with an error message.
    Error(String),
}

/// State for the hub UI.
pub struct Hub {
    /// Search engine owning the loaded cards.
    engine: SearchEngine,
    /// Project details popup.
    project_popup: ProjectPopup,
    /// Current catalog loading state.
    load_state: LoadState,
}

impl Default for Hub {
    fn default() -> Self {
        Self {
            engine: SearchEngine::new(Vec::new()),
            project_popup: ProjectPopup::default(),
            load_state: LoadState::Loading,
        }
    }
}

impl Hub {
    /// Attempts to load the card catalog.
    ///
    /// This is currently synchronous because the fixture is embedded at compile
    /// time. In the future it will be replaced by an async fetch, and this
    /// method can be called repeatedly from [`Hub::show`] until the load
    /// completes.
    fn load(&mut self) {
        match load_cards() {
            Ok(cards) => {
                self.engine = SearchEngine::new(cards);
                self.load_state = LoadState::Ready;
            }
            Err(error) => {
                self.load_state = LoadState::Error(error);
            }
        }
    }

    /// Renders the hub UI.
    pub fn show(&mut self, ui: &mut egui::Ui, search_bar: &mut SearchBar, search_open: &mut bool) {
        if matches!(self.load_state, LoadState::Loading) {
            self.load();
        }

        DottedBackground::builder()
            .spacing(24.0)
            .radius(1.0)
            .base_alpha(0.4)
            .fade_start(0.75)
            .build(ui);

        ui.add_space(24.0);

        let error_message = match &self.load_state {
            LoadState::Loading => {
                self.render_loading(ui);
                return;
            }
            LoadState::Error(error) => Some(error.clone()),
            LoadState::Ready => None,
        };

        if let Some(error) = error_message {
            self.render_error(ui, &error);
            return;
        }

        if *search_open {
            self.render_search_modal(ui, search_bar, search_open);
        }

        ui.add_space(24.0);
        let parsed = SearchEngine::parse_query(&search_bar.query);
        let cards = self.engine.search(&parsed);
        let card_data: Vec<CardData> = cards.into_iter().cloned().collect();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .content_margin(egui::Margin {
                left: 0,
                right: 48,
                top: 0,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.add_space(16.0);
                let actions = Grid.show(ui, &card_data);
                self.apply_card_actions(ui, actions, &card_data, search_bar);
            });

        let popup_actions = self.project_popup.show(ui);
        self.apply_card_actions(ui, popup_actions, &card_data, search_bar);
    }

    /// Renders a search modal with the search input and suggestions.
    fn render_search_modal(
        &mut self,
        ui: &mut egui::Ui,
        search_bar: &mut SearchBar,
        search_open: &mut bool,
    ) {
        let modal_width = 600.0;
        let query = search_bar.query.clone();
        let suggestions = self.engine.suggestions(&query);

        let modal_response =
            egui::Modal::new(egui::Id::new("hub_search_modal")).show(ui.ctx(), |ui| {
                ui.set_width(modal_width);
                let wants_close = search_bar.show(ui, &query, &suggestions);
                if wants_close {
                    *search_open = false;
                }
            });

        if modal_response.should_close() {
            *search_open = false;
        }
    }

    /// Renders a loading indicator while the catalog is being fetched.
    fn render_loading(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(64.0);
            ui.label(
                egui::RichText::new(crate::i18n::Hub::LOADING_TITLE)
                    .heading()
                    .size(24.0),
            );
            ui.add_space(8.0);
            ui.label(crate::i18n::Hub::LOADING_MESSAGE);
            ui.add_space(16.0);
            ui.spinner();
        });
    }

    /// Renders an error message with a retry action.
    fn render_error(&mut self, ui: &mut egui::Ui, error: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(64.0);
            ui.label(
                egui::RichText::new(crate::i18n::Hub::ERROR_TITLE)
                    .heading()
                    .size(24.0),
            );
            ui.add_space(8.0);
            ui.label(format!(
                "{}{}",
                crate::i18n::Hub::ERROR_MESSAGE_PREFIX,
                error
            ));
            ui.add_space(16.0);
            if ui.button(crate::i18n::Hub::RETRY).clicked() {
                self.load_state = LoadState::Loading;
            }
        });
    }

    /// Applies actions triggered by clicking interactive card elements.
    fn apply_card_actions(
        &mut self,
        _ui: &mut egui::Ui,
        actions: Vec<CardAction>,
        card_data: &[CardData],
        search_bar: &mut SearchBar,
    ) {
        for action in actions {
            match action {
                CardAction::Filter(filter) => {
                    let query = &mut search_bar.query;
                    if !query.is_empty() && !query.ends_with(' ') {
                        query.push(' ');
                    }
                    query.push_str(&filter);
                }
                CardAction::ClearSearch => {
                    search_bar.query.clear();
                }
                CardAction::OpenProject(index) => {
                    if let Some(data) = card_data.get(index) {
                        self.project_popup.open(data);
                    }
                }
            }
        }
    }
}
