use crate::domain::resource::Resource;
use eframe::egui::{self, Ui};
use uuid::Uuid;

pub fn show_resource_selector(ui: &mut Ui, selected: &mut Option<Uuid>, resources: &[Resource]) {
    egui::ComboBox::from_label("Assigned To")
        .selected_text(
            selected
                .and_then(|id| resources.iter().find(|r| r.id == id))
                .map(|r| r.name.clone())
                .unwrap_or_else(|| "Unassigned".to_string())
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(selected, None, "Unassigned");
            for resource in resources {
                ui.selectable_value(selected, Some(resource.id), &resource.name);
            }
        });
}