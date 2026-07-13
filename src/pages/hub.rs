//! Main hub UI component shown after a successful login.
//!
//! The card catalog is currently loaded from the embedded fixture managed by
//! [`crate::fixture`]. See that module for notes on the planned move to a
//! runtime data source.

use crate::components::{
    CardAction, CardData, DottedBackground, Grid, Keycap, ProjectPopup, SearchBar,
};
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
    /// Search control state.
    search_bar: SearchBar,
    /// Project details popup.
    project_popup: ProjectPopup,
    /// Current catalog loading state.
    load_state: LoadState,
}

impl Default for Hub {
    fn default() -> Self {
        Self {
            engine: SearchEngine::new(Vec::new()),
            search_bar: SearchBar::default(),
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
    pub fn show(&mut self, ui: &mut egui::Ui) {
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

        let inner_spacing = 20.0;
        let (columns, card_width) = Grid::column_metrics(ui.available_width());
        let search_width = if columns >= 2 {
            card_width * 2.0 + inner_spacing
        } else {
            card_width
        };

        let query = self.search_bar.query.clone();
        let suggestions = self.engine.suggestions(&query);
        ui.vertical_centered(|ui| {
            ui.set_max_width(search_width);
            self.search_bar.show(ui, &query, &suggestions);
        });

        let parsed = SearchEngine::parse_query(&self.search_bar.query);

        let mut focus_search = false;
        Keycap::builder()
            .keys(&[egui::Key::C, egui::Key::S][..])
            .execute(|| focus_search = true)
            .build(ui);

        if focus_search {
            ui.memory_mut(|mem| mem.request_focus(self.search_bar.id));
        }

        ui.add_space(24.0);
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
                self.apply_card_actions(ui, actions, &card_data);
            });

        let popup_actions = self.project_popup.show(ui);
        self.apply_card_actions(ui, popup_actions, &card_data);
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
    ) {
        for action in actions {
            match action {
                CardAction::Filter(filter) => {
                    let query = &mut self.search_bar.query;
                    if !query.is_empty() && !query.ends_with(' ') {
                        query.push(' ');
                    }
                    query.push_str(&filter);
                }
                CardAction::ClearSearch => {
                    self.search_bar.query.clear();
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
