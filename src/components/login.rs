//! Login form component for the Cadiotheka hub.

use crate::i18n;

/// State for the login form.
#[derive(Default)]
pub struct LoginForm {
    /// Username or email entered by the user.
    pub username: String,
    /// Password entered by the user.
    pub password: String,
}

impl LoginForm {
    /// Renders the centered login form.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_rect_before_wrap();

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let vertical_space = available.height() / 2.0;
            ui.add_space(vertical_space - 120.0);

            ui.heading(i18n::Login::TITLE);
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.label(i18n::Login::USERNAME_LABEL);
                ui.text_edit_singleline(&mut self.username);
            });

            ui.horizontal(|ui| {
                ui.label(i18n::Login::PASSWORD_LABEL);
                ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
            });

            ui.add_space(8.0);
            if ui.button(i18n::Login::BUTTON).clicked() {
                // TODO: handle login
            }
        });
    }
}
