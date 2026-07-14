//! Minimal Xodium accent tweak for egui.
//!
//! Keeps the default egui/eframe dark look, but swaps the blue accent color
//! for the orange used on <https://xodium.org>.

use egui::{Color32, Style, Visuals};

/// Xodium brand orange, taken from xodium.org's DaisyUI `halloween` primary.
pub const ORANGE: Color32 = Color32::from_rgb(249, 115, 22);

/// Returns the default egui dark style with the blue accent replaced by the
/// Xodium orange.
pub fn style() -> Style {
    Style {
        visuals: visuals(),
        ..Style::default()
    }
}

/// Returns default egui [`Visuals`] with blue accents swapped to orange.
pub fn visuals() -> Visuals {
    let mut visuals = Visuals::dark();

    // Swap blue accents for orange without touching text colors.
    visuals.hyperlink_color = ORANGE;
    visuals.selection.bg_fill = ORANGE.gamma_multiply(0.4);
    visuals.selection.stroke.color = ORANGE;

    // Widget outlines and active fills use orange; text stays the default color.
    visuals.widgets.hovered.bg_stroke.color = ORANGE;
    visuals.widgets.active.bg_fill = ORANGE;
    visuals.widgets.active.bg_stroke.color = ORANGE;
    visuals.widgets.open.bg_stroke.color = ORANGE;

    visuals
}
