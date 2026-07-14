//! Top navigation header for Cadiotheka.

use crate::components::SearchBar;
use crate::{components::Keycap, i18n};
use egui_phosphor_icons::icons::MAGNIFYING_GLASS;

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
    view: View,
}

impl Header {
    /// Returns the currently selected view.
    pub fn view(&self) -> View {
        self.view
    }

    /// Draw the top navigation header.
    pub fn show(&mut self, ui: &mut egui::Ui, search_bar: &mut SearchBar, search_open: &mut bool) {
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
                    Keycap::builder()
                        .keys(&[egui::Key::H][..])
                        .alt(true)
                        .execute(|| self.view = View::Hub)
                        .combine(true)
                        .attach(ui, &response);
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    search_button(ui, search_bar, search_open);

                    Keycap::builder()
                        .keys(&[egui::Key::S][..])
                        .ctrl(true)
                        .execute(|| {
                            *search_open = true;
                            search_bar.request_focus();
                        })
                        .build(ui);
                });
            });
        });
    }
}

/// Draws the search button with the magnifying glass icon and a `C + S` keycap
/// inside it.
fn search_button(
    ui: &mut egui::Ui,
    search_bar: &mut SearchBar,
    search_open: &mut bool,
) -> egui::Response {
    let icon_text = MAGNIFYING_GLASS.as_str();
    let keycap_label = "Ctrl + S";

    let icon_font = egui::FontId::proportional(16.0);
    let keycap_font = egui::FontId::proportional(10.0);
    let keycap_visuals = ui.visuals().widgets.inactive;

    let icon_galley = ui.ctx().fonts_mut(|f| {
        f.layout(
            icon_text.to_owned(),
            icon_font.clone(),
            ui.visuals().text_color(),
            f32::INFINITY,
        )
    });
    let keycap_galley = ui.ctx().fonts_mut(|f| {
        f.layout(
            keycap_label.to_owned(),
            keycap_font.clone(),
            keycap_visuals.fg_stroke.color,
            f32::INFINITY,
        )
    });

    let icon_size = icon_galley.size();
    let keycap_size = egui::vec2(keycap_galley.size().x + 10.0, 16.0);
    let content_spacing = 6.0;
    let content_size = egui::vec2(
        icon_size.x + content_spacing + keycap_size.x,
        icon_size.y.max(keycap_size.y),
    );
    let button_padding = ui.spacing().button_padding;
    let desired_size = content_size + 2.0 * button_padding;

    let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());
    let visuals = ui.style().interact(&response);

    ui.painter()
        .rect_filled(rect, visuals.corner_radius, visuals.bg_fill);
    ui.painter().rect_stroke(
        rect,
        visuals.corner_radius,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    let content_rect = rect.shrink2(button_padding);
    let icon_pos = egui::pos2(
        content_rect.left(),
        content_rect.center().y - icon_size.y / 2.0,
    );
    ui.painter()
        .galley(icon_pos, icon_galley, visuals.fg_stroke.color);

    let keycap_rect = egui::Rect::from_min_size(
        egui::pos2(
            icon_pos.x + icon_size.x + content_spacing,
            content_rect.center().y - keycap_size.y / 2.0,
        ),
        keycap_size,
    );
    ui.painter()
        .rect_filled(keycap_rect, 3.0, keycap_visuals.bg_fill);
    ui.painter().rect_stroke(
        keycap_rect,
        3.0,
        keycap_visuals.fg_stroke,
        egui::StrokeKind::Inside,
    );
    let keycap_text_pos = keycap_rect.center() - keycap_galley.size() * 0.5;
    ui.painter().galley(
        keycap_text_pos,
        keycap_galley,
        keycap_visuals.fg_stroke.color,
    );

    if response.clicked() {
        *search_open = true;
        search_bar.request_focus();
    }

    response
}
