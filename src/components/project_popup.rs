//! Project details popup for Cadiotheka.

use crate::components::card::CardData;

/// State and rendering for a project details popup.
#[derive(Debug, Default)]
pub struct ProjectPopup {
    /// Title of the project currently shown in the popup, if any.
    project_title: Option<String>,
}

impl ProjectPopup {
    /// Opens the popup for the given project.
    pub fn open(&mut self, data: &CardData) {
        self.project_title = Some(data.title.clone());
    }

    /// Closes the popup.
    pub fn close(&mut self) {
        self.project_title = None;
    }

    /// Returns whether the popup is currently open.
    pub fn is_open(&self) -> bool {
        self.project_title.is_some()
    }

    /// Draw the project popup as a centered modal window stub.
    ///
    /// Returns `true` when the popup is still open after this frame.
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(title) = self.project_title.clone() else {
            return false;
        };

        let mut open = true;
        let response = egui::Window::new(crate::i18n::ProjectPopup::TITLE)
            .collapsible(false)
            .resizable(false)
            .default_width(480.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{}: {}",
                        crate::i18n::ProjectPopup::PROJECT_LABEL,
                        title
                    ))
                    .heading(),
                );
                ui.add_space(16.0);
                ui.label(crate::i18n::ProjectPopup::STUB_MESSAGE);
                ui.add_space(24.0);
                if ui.button(crate::i18n::ProjectPopup::CLOSE).clicked() {
                    open = false;
                }
            });

        if response.is_none_or(|r| r.response.clicked_elsewhere()) {
            open &= !ui.input(|i| i.key_pressed(egui::Key::Escape));
        }

        if !open {
            self.close();
        }

        self.is_open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::card::CardData;
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
    fn popup_opens_with_project_title() {
        let mut popup = ProjectPopup::default();
        assert!(!popup.is_open());
        popup.open(&sample_card("Gear"));
        assert!(popup.is_open());
        assert_eq!(popup.project_title.as_deref(), Some("Gear"));
    }

    #[test]
    fn popup_close_clears_title() {
        let mut popup = ProjectPopup::default();
        popup.open(&sample_card("Gear"));
        popup.close();
        assert!(!popup.is_open());
        assert_eq!(popup.project_title, None);
    }
}
