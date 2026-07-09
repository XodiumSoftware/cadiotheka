//! The main Cadiotheka hub application state and UI.

use crate::components::hub::Hub;
use crate::components::login::LoginForm;

/// The Cadiotheka hub application.
#[derive(Default)]
pub struct CadiothekaApp {
    /// Login form state.
    login_form: LoginForm,
    /// Main hub UI, shown after a successful login.
    hub: Hub,
    /// Whether the user has logged in and should see the hub.
    is_logged_in: bool,
}

impl eframe::App for CadiothekaApp {
    /// Renders the hub UI each frame.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if self.is_logged_in {
                self.hub.show(ui);
            } else {
                let logged_in = self.login_form.show(ui);
                if logged_in {
                    self.is_logged_in = true;
                }
            }
        });
    }
}
