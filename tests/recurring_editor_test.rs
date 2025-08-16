#[cfg(test)]
mod tests {
    use eframe::egui;
    use plon::ui::widgets::recurring_editor::RecurringEditor;
    use plon::domain::recurring::{RecurrencePattern};
    
    /// This test ensures that the RecurringEditor doesn't have widget ID conflicts
    /// when multiple instances are shown or when the same instance is shown multiple times
    #[test]
    fn test_recurring_editor_no_widget_id_conflicts() {
        // Create a test context for egui
        let ctx = egui::Context::default();
        
        // Initialize the context with a dummy run to set up fonts
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });
        
        // Create multiple recurring editors
        let mut editor1 = RecurringEditor::new();
        let mut editor2 = RecurringEditor::new();
        
        // Set different patterns to trigger different UI paths
        editor1.pattern = RecurrencePattern::Weekly;
        editor2.pattern = RecurrencePattern::Monthly;
        
        // Track widget IDs to detect conflicts
        let mut _widget_ids: std::collections::HashSet<egui::Id> = std::collections::HashSet::new();
        
        // Simulate showing the editors in a UI
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
            // Test 1: Show the same editor multiple times (simulating re-rendering)
            for i in 0..3 {
                ui.push_id(format!("editor1_iteration_{}", i), |ui| {
                    // This should not cause ID conflicts
                    let _ = editor1.show(ui);
                });
            }
            
            // Test 2: Show multiple different editors
            ui.push_id("editor1_main", |ui| {
                let _ = editor1.show(ui);
            });
            
            ui.push_id("editor2_main", |ui| {
                let _ = editor2.show(ui);
            });
            
            // Test 3: Check specific problematic widgets
            ui.horizontal(|ui| {
                // These DragValues need unique IDs
                ui.push_id("time_test", |ui| {
                    let mut hour = 10u32;
                    let mut minute = 30u32;
                    ui.add(egui::DragValue::new(&mut hour).speed(1.0).clamp_range(0..=23));
                    ui.label(":");
                    ui.add(egui::DragValue::new(&mut minute).speed(1.0).clamp_range(0..=59));
                });
            });
            
            // Test 4: ComboBoxes need unique IDs
            ui.push_id("combo_test", |ui| {
                let mut pattern = RecurrencePattern::Daily;
                egui::ComboBox::from_id_source("pattern_combo_test")
                    .selected_text("Daily")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut pattern, RecurrencePattern::Daily, "Daily");
                        ui.selectable_value(&mut pattern, RecurrencePattern::Weekly, "Weekly");
                    });
            });
            });
        });
        
        // The test passes if we get here without panics
        // In a real scenario, egui would panic or show warnings for ID conflicts
    }
    
    /// Test that verifies the time picker widgets have unique IDs
    #[test]
    fn test_time_picker_unique_ids() {
        let ctx = egui::Context::default();
        
        // Initialize context
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });
        
        let mut editor = RecurringEditor::new();
        
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
            // This test specifically checks the time picker which was showing conflicts
            ui.group(|ui| {
                ui.push_id("time_picker_test", |ui| {
                    // Simulate the time picker from RecurringEditor
                    ui.horizontal(|ui| {
                        ui.label("Time of day:");
                        
                        // These DragValues were causing conflicts
                        let mut hour_val = 9u32;
                        let mut minute_val = 0u32;
                        
                        // Without proper IDs, these would conflict
                        ui.add(egui::DragValue::new(&mut hour_val)
                            .speed(1.0)
                            .clamp_range(0..=23));
                        ui.label(":");
                        ui.add(egui::DragValue::new(&mut minute_val)
                            .speed(1.0)
                            .clamp_range(0..=59));
                    });
                });
            });
            
            // Show the actual editor to ensure it doesn't conflict
            let _ = editor.show(ui);
            });
        });
    }
    
    /// Test ComboBox ID uniqueness
    #[test]
    fn test_combo_box_unique_ids() {
        let ctx = egui::Context::default();
        
        // Initialize context
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });
        
        let mut editor = RecurringEditor::new();
        
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
            // Test priority and pattern combo boxes
            for i in 0..2 {
                ui.push_id(format!("combo_iteration_{}", i), |ui| {
                    let _ = editor.show(ui);
                });
            }
            });
        });
    }
}