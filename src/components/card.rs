//! Card widget for Cadiotheka.

use crate::{platforms::Platform, tags::Tag, utils::Utils};

/// A URL pointing to a card's icon asset.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct IconUrl(pub String);

/// Data displayed on a content card.
#[derive(Debug, Clone)]
pub struct CardData {
    /// Card title.
    pub title: String,
    /// Author or creator name.
    pub author: String,
    /// Short description of the content.
    pub description: String,
    /// Categorized tags for the content.
    pub tags: Vec<Tag>,
    /// Supported platforms for the content.
    pub supported_platforms: Vec<Platform>,
    /// Download count.
    pub downloads: u64,
    /// Favorite count.
    pub favorites: u64,
    /// Official timestamp for when the card was published or updated.
    pub timestamp: time::OffsetDateTime,
    /// Optional icon URL (when absent, a colored placeholder is generated).
    pub icon_url: Option<IconUrl>,
}

/// An action triggered by interacting with a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardAction {
    /// Append a tag or platform filter to the search query.
    Filter(String),
}

/// State and rendering for a reusable card component.
#[derive(Default)]
pub struct Card;

impl Card {
    /// Draw the card component using the provided data.
    ///
    /// Returns a [`CardAction`] if the user clicked an interactive element.
    pub fn show(&self, ui: &mut egui::Ui, data: &CardData) -> Option<CardAction> {
        let mut action = None;

        let mut frame = egui::Frame::group(ui.style());
        frame.fill = ui.visuals().panel_fill;
        frame.shadow = egui::Shadow {
            offset: [0, 6],
            blur: 8,
            spread: 0,
            color: ui.visuals().window_shadow.color,
        };
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if data.icon_url.is_none() {
                        Self::show_icon_placeholder(ui, &data.title);
                    }
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(egui::RichText::new(&data.title).strong().size(18.0));
                        ui.label(" by ");
                        ui.label(egui::RichText::new(&data.author).strong());
                    });
                });
                ui.separator();
                ui.label(&data.description);
                ui.separator();
                ui.horizontal(|ui| {
                    for tag in &data.tags {
                        let label = format!("#{}", tag.label());
                        let response = ui.add(
                            egui::Button::new(egui::RichText::new(tag.label()).size(11.0))
                                .fill(ui.visuals().widgets.inactive.bg_fill)
                                .small(),
                        );
                        if response.clicked() {
                            action = Some(CardAction::Filter(label));
                        }
                    }
                    ui.separator();
                    for platform in &data.supported_platforms {
                        let label = format!("#{}", platform.label());
                        let response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(platform.label())
                                    .size(11.0)
                                    .color(platform.color()),
                            )
                            .fill(ui.visuals().widgets.inactive.bg_fill)
                            .small(),
                        );
                        if response.clicked() {
                            action = Some(CardAction::Filter(label));
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("⬇ {}", Utils::format_number(data.downloads)))
                        .on_hover_text(format!(
                            "{} downloads",
                            Utils::format_number_full(data.downloads)
                        ));
                    ui.label(format!("❤ {}", Utils::format_number(data.favorites)))
                        .on_hover_text(format!(
                            "{} favorites",
                            Utils::format_number_full(data.favorites)
                        ));
                    ui.label(format!("🕒 {}", Utils::format_time_ago(data.timestamp)))
                        .on_hover_text(format!(
                            "Updated {}",
                            Utils::format_time_full(data.timestamp)
                        ));
                });
            });
        });

        action
    }

    fn show_icon_placeholder(ui: &mut egui::Ui, title: &str) {
        let first_letter = Self::placeholder_letter(title);
        let color = Self::placeholder_color(title);

        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, color);

        let font_id = egui::FontId::proportional(24.0);
        let text_color = egui::Color32::WHITE;
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            first_letter,
            font_id,
            text_color,
        );
    }

    fn placeholder_letter(title: &str) -> String {
        title
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string()
    }

    fn placeholder_color(title: &str) -> egui::Color32 {
        let palette: [egui::Color32; 8] = [
            egui::Color32::from_rgb(239, 68, 68),  // red
            egui::Color32::from_rgb(249, 115, 22), // orange
            egui::Color32::from_rgb(234, 179, 8),  // yellow
            egui::Color32::from_rgb(34, 197, 94),  // green
            egui::Color32::from_rgb(6, 182, 212),  // cyan
            egui::Color32::from_rgb(59, 130, 246), // blue
            egui::Color32::from_rgb(168, 85, 247), // purple
            egui::Color32::from_rgb(236, 72, 153), // pink
        ];

        // Hash the full title so cards with the same length but different
        // content get different placeholder colors.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(title, &mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        palette[(hash as usize) % palette.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_url_serializes_transparently() {
        let url = IconUrl("https://example.com/icon.svg".to_owned());
        let json = serde_json::to_string(&url).unwrap();
        assert_eq!(json, "\"https://example.com/icon.svg\"");

        let decoded: IconUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, url);
    }

    #[test]
    fn placeholder_letter_uppercases_first_char() {
        assert_eq!(Card::placeholder_letter("Blender"), "B");
        assert_eq!(Card::placeholder_letter("freecad"), "F");
        assert_eq!(Card::placeholder_letter(""), "?");
    }

    #[test]
    fn placeholder_color_varies_by_title_content() {
        let a = Card::placeholder_color("Blender");
        let b = Card::placeholder_color("FreeCAD");
        // Same length but different content should usually differ. The hash
        // space is small (8 colors), so exact equality is allowed but unlikely.
        assert_ne!(
            a, b,
            "different titles of the same length should get different colors"
        );
    }

    #[test]
    fn placeholder_color_is_deterministic_for_same_title() {
        let a = Card::placeholder_color("abc");
        let b = Card::placeholder_color("abc");
        assert_eq!(a, b);
    }
}
