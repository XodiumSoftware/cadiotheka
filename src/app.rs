//! The main Cadiotheka hub application state and UI.

use crate::components::{Header, View};
use crate::pages::{Hub, LoginForm};

/// The Cadiotheka hub application.
#[derive(Default)]
pub struct CadiothekaApp {
    /// Login form state.
    login_form: LoginForm,
    /// Main hub UI, shown after a successful login.
    hub: Hub,
    /// Whether the user has logged in and should see the hub.
    is_logged_in: bool,
    /// Top navigation header.
    header: Header,
    /// Currently selected view in the hub.
    view: View,
}

impl eframe::App for CadiothekaApp {
    /// Renders the hub UI each frame.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.is_logged_in {
            self.header.show(ui);
            self.view = self.header.view;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            if self.is_logged_in {
                match self.view {
                    View::Hub => self.hub.show(ui),
                }
            } else if self.login_form.show(ui) {
                self.is_logged_in = true;
            }
        });
    }
}
