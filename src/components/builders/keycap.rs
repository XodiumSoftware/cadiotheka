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
    combine: bool,
    ctrl: bool,
    alt: bool,
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
            combine: false,
            ctrl: false,
            alt: false,
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

    /// Enables inline rendering of the keycap sequence in the current layout.
    ///
    /// Useful for showing a keycap next to a button or label without a hover
    /// tooltip.
    pub fn inline(mut self, inline: bool) -> Self {
        self.tooltip = inline;
        self
    }

    /// Renders all configured keys inside a single combined keycap.
    ///
    /// When disabled (default), each key gets its own keycap separated by
    /// [`Self::separator`].
    pub fn combine(mut self, combine: bool) -> Self {
        self.combine = combine;
        self
    }

    /// Requires the Ctrl modifier to be held for the chord to trigger.
    pub fn ctrl(mut self, ctrl: bool) -> Self {
        self.ctrl = ctrl;
        self
    }

    /// Requires the Alt modifier to be held for the chord to trigger.
    pub fn alt(mut self, alt: bool) -> Self {
        self.alt = alt;
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
        let label = self.combined_label();

        if self.check_chord(ui)
            && let Some(mut f) = self.execute
        {
            f();
        }

        let size = self.size;
        let rounding = self.rounding;
        let font_size = self.font_size;
        response.clone().on_hover_ui(move |ui| {
            Self::draw_key_static(ui, &label, size, rounding, font_size);
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

    fn combined_label(&self) -> String {
        let spaced_separator = format!(" {} ", self.separator);
        let keys = self
            .keys
            .iter()
            .map(|key| key.name())
            .collect::<Vec<_>>()
            .join(&spaced_separator);

        let mut prefix = String::new();
        if self.ctrl {
            prefix.push_str("Ctrl");
        }
        if self.alt {
            if !prefix.is_empty() {
                prefix.push_str(&spaced_separator);
            }
            prefix.push_str("Alt");
        }

        if prefix.is_empty() {
            keys
        } else if keys.is_empty() {
            prefix
        } else {
            format!("{prefix}{spaced_separator}{keys}")
        }
    }

    fn check_chord(&self, ui: &egui::Ui) -> bool {
        if self.keys.is_empty() {
            return false;
        }
        let last = self.keys.len() - 1;
        ui.input(|i| {
            let modifiers_ok = (!self.ctrl || i.modifiers.ctrl) && (!self.alt || i.modifiers.alt);
            modifiers_ok
                && self.keys[..last].iter().all(|key| i.key_down(*key))
                && i.key_pressed(self.keys[last])
        })
    }

    fn draw_sequence(&self, ui: &mut egui::Ui) {
        if self.combine {
            let label = self.combined_label();
            self.draw_key(ui, &label);
        } else {
            let mut iter = self.keys.iter().peekable();
            while let Some(key) = iter.next() {
                self.draw_key(ui, key.name());
                if iter.peek().is_some() {
                    ui.label(self.separator);
                }
            }
        }
    }

    fn draw_key_static(ui: &mut egui::Ui, label: &str, size: f32, rounding: f32, font_size: f32) {
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

    fn draw_key(&self, ui: &mut egui::Ui, label: &str) {
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
