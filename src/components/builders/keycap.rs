//! Keycap widget for Cadiotheka.

/// Configures and renders a keyboard-style keycap sequence.
///
/// Start with [`Keycap::builder`], chain any adjustments, then call
/// [`Keycap::build`] to draw.
pub struct Keycap<'a> {
    size: f32,
    rounding: f32,
    font_size: f32,
    separator: &'static str,
    tooltip: bool,
    keys: Vec<egui::Key>,
    execute: Option<Box<dyn FnMut() + 'a>>,
}

impl Default for Keycap<'_> {
    fn default() -> Self {
        Self {
            size: 20.0,
            rounding: 4.0,
            font_size: 12.0,
            separator: "+",
            tooltip: false,
            keys: Vec::new(),
            execute: None,
        }
    }
}

impl<'a> Keycap<'a> {
    /// Starts building a keycap with default settings.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the keycap keys to render.
    pub fn keys(mut self, keys: &[egui::Key]) -> Self {
        self.keys = keys.to_vec();
        self
    }

    /// Enables or disables tooltip-style rendering.
    ///
    /// When enabled, the keys are rendered inside a horizontal layout.
    pub fn tooltip(mut self, tooltip: bool) -> Self {
        self.tooltip = tooltip;
        self
    }

    /// Sets a closure to run when the configured key chord is triggered.
    pub fn execute(mut self, f: impl FnMut() + 'a) -> Self {
        self.execute = Some(Box::new(f));
        self
    }

    /// Sets a new separator string between keycaps.
    pub fn separator(mut self, separator: &'static str) -> Self {
        self.separator = separator;
        self
    }

    /// Sets a new size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets a new rounding.
    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    /// Sets a new font size.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Checks the configured chord, runs `execute` if triggered, and attaches
    /// the keycap tooltip to the given response's hover event.
    pub fn attach(self, ui: &mut egui::Ui, response: &egui::Response) {
        if self.check_chord(ui)
            && let Some(mut f) = self.execute
        {
            f();
        }

        let keys = self.keys.clone();
        let size = self.size;
        let rounding = self.rounding;
        let font_size = self.font_size;
        let separator = self.separator;
        response.clone().on_hover_ui(move |ui| {
            ui.horizontal(|ui| {
                let mut iter = keys.iter().peekable();
                while let Some(key) = iter.next() {
                    Self::draw_key_static(ui, *key, size, rounding, font_size);
                    if iter.peek().is_some() {
                        ui.label(separator);
                    }
                }
            });
        });
    }

    /// Checks whether the configured key chord is currently pressed, runs the
    /// configured execute closure when triggered, and renders the keycaps if
    /// tooltip mode is enabled.
    ///
    /// A chord is recognized when all keys except the last are held down and the
    /// last key is pressed on this frame.
    pub fn build(mut self, ui: &mut egui::Ui) -> bool {
        let triggered = self.check_chord(ui);
        if triggered && let Some(ref mut f) = self.execute {
            f();
        }
        if self.tooltip {
            ui.horizontal(|ui| self.draw_sequence(ui));
        }
        triggered
    }

    fn check_chord(&self, ui: &egui::Ui) -> bool {
        if self.keys.len() < 2 {
            return false;
        }
        let last = self.keys.len() - 1;
        ui.input(|i| {
            self.keys[..last].iter().all(|key| i.key_down(*key)) && i.key_pressed(self.keys[last])
        })
    }

    fn draw_sequence(&self, ui: &mut egui::Ui) {
        let mut iter = self.keys.iter().peekable();
        while let Some(key) = iter.next() {
            self.draw_key(ui, key);
            if iter.peek().is_some() {
                ui.label(self.separator);
            }
        }
    }

    fn draw_key_static(
        ui: &mut egui::Ui,
        key: egui::Key,
        size: f32,
        rounding: f32,
        font_size: f32,
    ) {
        let label = key.name();
        let bg = ui.visuals().widgets.inactive.bg_fill;
        let stroke = ui.visuals().widgets.inactive.fg_stroke;
        let text_color = ui.visuals().widgets.inactive.fg_stroke.color;
        let padding = 8.0f32;
        let font_id = egui::FontId::proportional(font_size);
        let galley = ui.ctx().fonts_mut(|f| {
            f.layout(
                label.to_string(),
                font_id.clone(),
                text_color,
                f32::INFINITY,
            )
        });
        let text_width = galley.size().x.max(size - padding);
        let width = text_width + padding;
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(width, size), egui::Sense::hover());

        ui.painter().rect_filled(rect, rounding, bg);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, egui::StrokeKind::Inside);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font_id,
            text_color,
        );
    }

    fn draw_key(&self, ui: &mut egui::Ui, key: &egui::Key) {
        let label = key.name();
        let bg = ui.visuals().widgets.inactive.bg_fill;
        let stroke = ui.visuals().widgets.inactive.fg_stroke;
        let text_color = ui.visuals().widgets.inactive.fg_stroke.color;
        let padding = 8.0f32;
        let font_id = egui::FontId::proportional(self.font_size);
        let galley = ui.ctx().fonts_mut(|f| {
            f.layout(
                label.to_string(),
                font_id.clone(),
                text_color,
                f32::INFINITY,
            )
        });
        let text_width = galley.size().x.max(self.size - padding);
        let width = text_width + padding;
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(width, self.size), egui::Sense::hover());

        ui.painter().rect_filled(rect, self.rounding, bg);
        ui.painter()
            .rect_stroke(rect, self.rounding, stroke, egui::StrokeKind::Inside);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font_id,
            text_color,
        );
    }
}
