//! Small reusable utilities for the Cadiotheka hub.

/// Spacing between grid dots in logical pixels.
const DOT_SPACING: f32 = 24.0;
/// Radius of each grid dot in logical pixels.
const DOT_RADIUS: f32 = 1.0;
/// How far from the rectangle center the dots remain at full brightness (0–1).
const FADE_START: f32 = 0.75;

/// Reusable utility helpers for the application.
#[derive(Default)]
pub struct Utils;

impl Utils {
    /// Draws a dotted grid background covering the given UI area.
    ///
    /// Dots are brighter near the center and fade toward the edges of the area.
    pub fn dotted_background(ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let color = ui.visuals().widgets.noninteractive.fg_stroke.color;
        let base_alpha = 0.4;
        let center = rect.center();
        let half_size = egui::vec2(rect.width() / 2.0, rect.height() / 2.0);

        let mut x = rect.left() + DOT_SPACING / 2.0;
        while x < rect.right() {
            let mut y = rect.top() + DOT_SPACING / 2.0;
            while y < rect.bottom() {
                let pos = egui::pos2(x, y);
                let distance = ((pos.x - center.x).abs() / half_size.x)
                    .max((pos.y - center.y).abs() / half_size.y);
                let fade = 1.0 - ((distance - FADE_START) / (1.0 - FADE_START)).clamp(0.0, 1.0);
                let dot_color = color.gamma_multiply(base_alpha * fade);

                ui.painter().circle_filled(pos, DOT_RADIUS, dot_color);
                y += DOT_SPACING;
            }
            x += DOT_SPACING;
        }
    }
}
