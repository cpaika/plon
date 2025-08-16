use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Test App",
        options,
        Box::new(|_cc| Box::new(TestApp::default())),
    )
}

#[derive(Default)]
struct TestApp {}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plon is working!");
            ui.label("The GUI framework is functioning correctly.");
        });
    }
}