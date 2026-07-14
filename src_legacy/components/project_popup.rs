//! Project details popup for Cadiotheka.

use crate::components::CardAction;
use crate::components::Keycap;
use crate::components::card::{Card, CardData};
use crate::utils::Utils;

/// State and rendering for a project details popup.
#[derive(Debug, Default)]
pub struct ProjectPopup {
    /// Data of the project currently shown in the popup, if any.
    project: Option<CardData>,
}

impl ProjectPopup {
    /// Opens the popup for the given project.
    pub fn open(&mut self, data: &CardData) {
        self.project = Some(data.clone());
    }

    /// Closes the popup.
    pub fn close(&mut self) {
        self.project = None;
    }

    /// Returns whether the popup is currently open.
    pub fn is_open(&self) -> bool {
        self.project.is_some()
    }

    /// Draw the project popup as a centered modal window.
    ///
    /// Returns any [`CardAction`]s triggered by clicking interactive popup
    /// elements (e.g., tags).
    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<CardAction> {
        let Some(data) = self.project.clone() else {
            return Vec::new();
        };

        let mut actions = Vec::new();
        let title = data.title.clone();
        let modal_id = egui::Id::new("project_popup");
        let response = egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&title).heading());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_response = ui.button(crate::i18n::ProjectPopup::CLOSE);
                    Keycap::builder()
                        .keys(&[egui::Key::Escape])
                        .attach(ui, &close_response);
                    if close_response.clicked() {
                        self.close();
                    }
                });
            });
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if let Some(icon_url) = &data.icon_url {
                    Card::show_icon_image(ui, &icon_url.0);
                } else {
                    Card::show_icon_placeholder(ui, &data.title);
                }
                ui.vertical(|ui| {
                    ui.label(&data.description);
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for tag in &data.tags {
                        let label = format!("#{}", tag.label());
                        let response = ui.add(
                            egui::Button::new(egui::RichText::new(tag.label()).size(11.0))
                                .fill(ui.visuals().widgets.inactive.bg_fill)
                                .small(),
                        );
                        if response.clicked() {
                            actions.push(CardAction::Filter(label));
                            self.close();
                        }
                    }
                    ui.label(format!("❤ {}", Utils::format_number(data.favorites)))
                        .on_hover_text(format!(
                            "{} favorites",
                            Utils::format_number_full(data.favorites)
                        ));
                    ui.label(format!("⬇ {}", Utils::format_number(data.downloads)))
                        .on_hover_text(format!(
                            "{} downloads",
                            Utils::format_number_full(data.downloads)
                        ));
                });
            });
        });

        if response.should_close() {
            self.close();
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::card::{CardData, IconUrl};
    use crate::platforms::Platform;
    use crate::tags::Tag;

    fn sample_card(title: &str) -> CardData {
        CardData {
            title: title.to_owned(),
            author: "Author".to_owned(),
            description: "Description".to_owned(),
            tags: vec![Tag::Model3d],
            supported_platforms: vec![Platform::Blender],
            downloads: 0,
            favorites: 0,
            timestamp: time::macros::datetime!(2024-01-01 00:00:00 UTC),
            icon_url: None,
        }
    }

    #[test]
    fn popup_opens_with_project_data() {
        let mut popup = ProjectPopup::default();
        assert!(!popup.is_open());
        popup.open(&sample_card("Gear"));
        assert!(popup.is_open());
        assert_eq!(popup.project.as_ref().unwrap().title, "Gear");
    }

    #[test]
    fn popup_close_clears_project() {
        let mut popup = ProjectPopup::default();
        popup.open(&sample_card("Gear"));
        popup.close();
        assert!(!popup.is_open());
        assert_eq!(popup.project, None);
    }

    #[test]
    fn popup_preserves_icon_url_and_description() {
        let mut card = sample_card("Gear");
        card.description = "A detailed gear project.".to_owned();
        card.icon_url = Some(IconUrl("https://example.com/gear.svg".to_owned()));

        let mut popup = ProjectPopup::default();
        popup.open(&card);

        let stored = popup.project.as_ref().unwrap();
        assert_eq!(stored.description, "A detailed gear project.");
        assert_eq!(
            stored.icon_url.as_ref().unwrap().0,
            "https://example.com/gear.svg"
        );
    }
}
