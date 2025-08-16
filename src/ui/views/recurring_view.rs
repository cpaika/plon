use eframe::egui::{self, Ui};

pub struct RecurringView {}

impl RecurringView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("Recurring Tasks");
        
        ui.label("Configure recurring tasks that automatically generate on a schedule.");
        
        if ui.button("+ New Recurring Task").clicked() {
            // TODO: Open recurring task editor
        }
        
        ui.separator();
        
        ui.label("Active Recurring Tasks:");
        // TODO: List recurring tasks
    }
}