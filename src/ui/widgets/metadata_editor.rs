use eframe::egui::Ui;
use std::collections::HashMap;

pub fn show_metadata_editor(ui: &mut Ui, metadata: &mut HashMap<String, String>) {
    ui.label("Metadata:");

    let mut to_remove = Vec::new();

    for (key, value) in metadata.iter_mut() {
        ui.horizontal(|ui| {
            ui.label(key);
            ui.text_edit_singleline(value);
            if ui.small_button("❌").clicked() {
                to_remove.push(key.clone());
            }
        });
    }

    for key in to_remove {
        metadata.remove(&key);
    }

    if ui.button("+ Add Metadata").clicked() {
        metadata.insert("new_key".to_string(), "value".to_string());
    }
}
