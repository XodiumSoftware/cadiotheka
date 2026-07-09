//! Sort bar widget for the Cadiotheka hub.

use crate::i18n;
use egui_phosphor_icons::icons;

/// Sort direction for a selected criterion.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order (lowest first).
    #[default]
    Ascending,
    /// Descending order (highest first).
    Descending,
}

impl SortOrder {
    /// Returns the Phosphor icon for this order.
    pub fn icon(self) -> egui::RichText {
        match self {
            Self::Ascending => icons::SORT_ASCENDING.regular(),
            Self::Descending => icons::SORT_DESCENDING.regular(),
        }
    }

    /// Toggles between ascending and descending.
    pub const fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Sort criterion selected by the user.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by download count.
    #[default]
    Downloads,
    /// Sort by favorite count.
    Favorites,
    /// Sort by timestamp.
    Newest,
}

impl SortBy {
    /// All available sort options in display order.
    pub const fn all() -> [Self; 3] {
        [Self::Downloads, Self::Favorites, Self::Newest]
    }

    /// User-facing label for this sort option.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Downloads => i18n::SortBar::DOWNLOADS,
            Self::Favorites => i18n::SortBar::FAVORITES,
            Self::Newest => i18n::SortBar::NEWEST,
        }
    }
}

/// Combined sort selection used by consumers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SortSelection {
    /// Field to sort by.
    pub by: SortBy,
    /// Direction of the sort.
    pub order: SortOrder,
}

/// State and rendering for a sort control bar.
#[derive(Default)]
pub struct SortBar {
    /// Currently selected sort criterion.
    pub sort_by: SortBy,
    /// Current sort direction.
    pub order: SortOrder,
}

impl SortBar {
    /// Draw the sort bar inside a card-like container.
    ///
    /// Returns the sort selection for this frame.
    pub fn show(&mut self, ui: &mut egui::Ui) -> SortSelection {
        let margin = 24.0;
        let mut frame = egui::Frame::group(ui.style());
        frame.fill = frame.fill.gamma_multiply(0.65);
        let spacing = ui.spacing().item_spacing.x;
        let separator_color = ui.visuals().widgets.noninteractive.fg_stroke.color;

        egui::Frame::new()
            .inner_margin(egui::Margin::same(margin as i8))
            .show(ui, |ui| {
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(i18n::SortBar::LABEL).strong());

                        let options = SortBy::all();
                        for (i, option) in options.iter().enumerate() {
                            let is_selected = self.sort_by == *option;
                            let text = if is_selected {
                                egui::RichText::new(option.label()).strong()
                            } else {
                                egui::RichText::new(option.label())
                            };

                            let response = ui.selectable_label(is_selected, text);
                            if is_selected {
                                ui.label(self.order.icon().size(14.0));
                            }

                            if response.clicked() {
                                if self.sort_by == *option {
                                    self.order = self.order.toggle();
                                } else {
                                    self.sort_by = *option;
                                    self.order = SortOrder::Descending;
                                }
                            }

                            if i + 1 < options.len() {
                                ui.add_space(spacing);
                                let separator_size = egui::vec2(1.0, ui.available_height());
                                let (rect, _response) =
                                    ui.allocate_exact_size(separator_size, egui::Sense::hover());
                                ui.painter().line_segment(
                                    [rect.left_top(), rect.left_bottom()],
                                    egui::Stroke::new(1.0, separator_color),
                                );
                                ui.add_space(spacing);
                            }
                        }
                    });
                });
            });

        SortSelection {
            by: self.sort_by,
            order: self.order,
        }
    }
}
