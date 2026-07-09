fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Cadiotheka"),
        ..Default::default()
    };

    eframe::run_native(
        "Cadiotheka",
        options,
        Box::new(|_cc| Ok(Box::new(CadiothekaApp::default()))),
    )
    .expect("failed to start Cadiotheka");
}

#[derive(Default)]
struct CadiothekaApp {
    name: String,
    counter: i32,
}

impl eframe::App for CadiothekaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Welcome to Cadiotheka");
            ui.horizontal(|ui| {
                ui.label("Your name:");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.horizontal(|ui| {
                ui.label("Counter:");
                if ui.button("-").clicked() {
                    self.counter -= 1;
                }
                ui.label(self.counter.to_string());
                if ui.button("+").clicked() {
                    self.counter += 1;
                }
            });
        });
    }
}
