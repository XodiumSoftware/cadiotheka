//! Dotted grid background component for Cadiotheka.

/// Configures and renders a dotted grid background.
///
/// Start with [`DottedBackground::builder`], chain any adjustments, then call
/// [`DottedBackground::build`] to draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DottedBackground {
    spacing: f32,
    radius: f32,
    base_alpha: f32,
    fade_start: f32,
}

impl Default for DottedBackground {
    fn default() -> Self {
        Self {
            spacing: 24.0,
            radius: 1.0,
            base_alpha: 0.4,
            fade_start: 0.75,
        }
    }
}

impl DottedBackground {
    /// Starts building a dotted background with default settings.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets a new spacing.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets a new radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets a new base alpha.
    pub fn base_alpha(mut self, alpha: f32) -> Self {
        self.base_alpha = alpha;
        self
    }

    /// Sets a new fade start.
    pub fn fade_start(mut self, start: f32) -> Self {
        self.fade_start = start.clamp(0.0, 1.0);
        self
    }

    /// Draws the configured dotted background into the given UI area.
    ///
    /// Dots are brighter near the center and fade toward the edges.
    pub fn build(self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let color = ui.visuals().widgets.noninteractive.fg_stroke.color;
        let center = rect.center();
        let half_size = egui::vec2(rect.width() / 2.0, rect.height() / 2.0);

        let mut x = rect.left() + self.spacing / 2.0;
        while x < rect.right() {
            let mut y = rect.top() + self.spacing / 2.0;
            while y < rect.bottom() {
                let pos = egui::pos2(x, y);
                let distance = ((pos.x - center.x).abs() / half_size.x)
                    .max((pos.y - center.y).abs() / half_size.y);
                let fade =
                    1.0 - ((distance - self.fade_start) / (1.0 - self.fade_start)).clamp(0.0, 1.0);
                let dot_color = color.gamma_multiply(self.base_alpha * fade);

                ui.painter().circle_filled(pos, self.radius, dot_color);
                y += self.spacing;
            }
            x += self.spacing;
        }
    }
}
