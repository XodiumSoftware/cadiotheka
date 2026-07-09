//! The main Cadiotheka hub application state and UI.

use crate::components::hub::Hub;
use crate::components::login::LoginForm;
use crate::i18n;

/// The currently selected main view.
#[derive(Default, PartialEq, Eq)]
enum View {
    /// The main hub dashboard.
    #[default]
    Hub,
}

/// The Cadiotheka hub application.
#[derive(Default)]
pub struct CadiothekaApp {
    /// Login form state.
    login_form: LoginForm,
    /// Main hub UI, shown after a successful login.
    hub: Hub,
    /// Whether the user has logged in and should see the hub.
    is_logged_in: bool,
    /// Currently selected view in the hub.
    view: View,
}

impl eframe::App for CadiothekaApp {
    /// Renders the hub UI each frame.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.is_logged_in {
            self.render_header(ui);
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

impl CadiothekaApp {
    /// Renders the top navigation header.
    fn render_header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("hub_header").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(i18n::Hub::HEADER);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.view, View::Hub, i18n::Hub::HUB_BUTTON);
                });
            });
        });
    }
}
