//! Top navigation header for Cadiotheka.

use crate::i18n;

/// Keyboard shortcut to open the Hub view.
pub struct HubShortcut;

impl HubShortcut {
    /// Checks for the `c + h` chord and returns `true` when triggered.
    pub fn check(ui: &egui::Ui) -> bool {
        ui.input(|i| i.key_down(egui::Key::C) && i.key_pressed(egui::Key::H))
    }

    /// Draws a single keycap with the given label.
    fn keycap(ui: &mut egui::Ui, label: &str) {
        let size = 20.0;
        let rounding = 4.0;
        let bg = ui.visuals().widgets.inactive.bg_fill;
        let stroke = ui.visuals().widgets.inactive.fg_stroke;
        let text_color = ui.visuals().widgets.inactive.fg_stroke.color;

        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        ui.painter().rect_filled(rect, rounding, bg);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, egui::StrokeKind::Inside);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            text_color,
        );
    }

    /// Renders the shortcut tooltip with keycaps.
    pub fn tooltip(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Open Hub");
            Self::keycap(ui, "C");
            ui.label("+");
            Self::keycap(ui, "H");
        });
    }
}

/// The currently selected main view.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum View {
    /// The main hub dashboard.
    #[default]
    Hub,
}

/// Top navigation header for the main application window.
#[derive(Default)]
pub struct Header {
    /// Currently selected view in the hub.
    pub view: View,
}

impl Header {
    /// Draw the top navigation header.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if HubShortcut::check(ui) {
            self.view = View::Hub;
        }

        egui::Panel::top("hub_header").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(i18n::Header::HEADER).strong());
                ui.separator();
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let response = ui.selectable_value(
                        &mut self.view,
                        View::Hub,
                        format!("{} {}", i18n::Header::HUB_ICON, i18n::Header::HUB_BUTTON),
                    );
                    response.on_hover_ui(|ui| HubShortcut::tooltip(ui));
                });
            });
        });
    }
}
