use eframe::egui::{self, Ui};
use crate::services::RecurringService;
use crate::domain::recurring::RecurringTaskTemplate;
use crate::ui::widgets::recurring_editor::RecurringEditor;
use uuid::Uuid;
use anyhow::Result;

pub struct RecurringView {
    pub templates: Vec<RecurringTaskTemplate>,
    pub show_editor: bool,
    pub editor: RecurringEditor,
    selected_template: Option<Uuid>,
    error_message: Option<String>,
}

impl RecurringView {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            show_editor: false,
            editor: RecurringEditor::new(),
            selected_template: None,
            error_message: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, service: Option<&RecurringService>) {
        ui.heading("🔄 Recurring Tasks");
        
        ui.label("Configure recurring tasks that automatically generate on a schedule.");
        
        ui.separator();
        
        // Show error message if any
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, error);
            if ui.button("Dismiss").clicked() {
                self.error_message = None;
            }
            ui.separator();
        }
        
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ New Recurring Task").clicked() {
                self.show_editor = true;
                self.editor.reset();
            }
            
            if ui.button("🔄 Refresh").clicked() {
                if let Some(svc) = service {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    match rt.block_on(self.load_templates(svc)) {
                        Ok(_) => {},
                        Err(e) => self.error_message = Some(format!("Failed to load templates: {}", e)),
                    }
                }
            }
        });
        
        ui.separator();
        
        // Show editor if open
        if self.show_editor {
            ui.group(|ui| {
                ui.heading("Create Recurring Task");
                if self.editor.show(ui) {
                    if let Some(svc) = service {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        match rt.block_on(self.create_template(svc)) {
                            Ok(_) => {
                                self.show_editor = false;
                                self.editor.reset();
                                // Reload templates
                                let _ = rt.block_on(self.load_templates(svc));
                            },
                            Err(e) => self.error_message = Some(format!("Failed to create template: {}", e)),
                        }
                    }
                }
                
                if ui.button("Cancel").clicked() {
                    self.show_editor = false;
                    self.editor.reset();
                }
            });
            
            ui.separator();
        }
        
        // List templates
        ui.label(format!("Active Recurring Tasks ({})", self.templates.len()));
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            let templates = self.templates.clone();
            for template in &templates {
                self.show_template_card(ui, template, service);
                ui.separator();
            }
        });
        
        if self.templates.is_empty() {
            ui.label("No recurring tasks configured. Click 'New Recurring Task' to create one.");
        }
    }
    
    fn show_template_card(&mut self, ui: &mut Ui, template: &RecurringTaskTemplate, service: Option<&RecurringService>) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Status indicator
                if template.active {
                    ui.colored_label(egui::Color32::GREEN, "●");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "●");
                }
                
                // Title
                ui.heading(&template.title);
                
                // Actions
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑️").on_hover_text("Delete").clicked() {
                        if let Some(svc) = service {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            match rt.block_on(self.delete_template(svc, template.id)) {
                                Ok(_) => {
                                    let _ = rt.block_on(self.load_templates(svc));
                                },
                                Err(e) => self.error_message = Some(format!("Failed to delete: {}", e)),
                            }
                        }
                    }
                    
                    let toggle_text = if template.active { "⏸️" } else { "▶️" };
                    let toggle_hover = if template.active { "Deactivate" } else { "Activate" };
                    if ui.button(toggle_text).on_hover_text(toggle_hover).clicked() {
                        if let Some(svc) = service {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            match rt.block_on(self.toggle_template(svc, template.id)) {
                                Ok(_) => {
                                    let _ = rt.block_on(self.load_templates(svc));
                                },
                                Err(e) => self.error_message = Some(format!("Failed to toggle: {}", e)),
                            }
                        }
                    }
                });
            });
            
            ui.label(&template.description);
            
            ui.horizontal(|ui| {
                ui.label(format!("Pattern: {:?}", template.recurrence_rule.pattern));
                ui.label(format!("Interval: {}", template.recurrence_rule.interval));
                
                if !template.recurrence_rule.days_of_week.is_empty() {
                    let days: Vec<String> = template.recurrence_rule.days_of_week.iter()
                        .map(|d| format!("{:?}", d).chars().take(3).collect())
                        .collect();
                    ui.label(format!("Days: {}", days.join(", ")));
                }
            });
            
            ui.horizontal(|ui| {
                ui.label(format!("Priority: {:?}", template.priority));
                
                if let Some(hours) = template.estimated_hours {
                    ui.label(format!("Est: {}h", hours));
                }
                
                ui.label(format!("Generated: {} times", template.recurrence_rule.occurrences_count));
            });
            
            if let Some(next) = template.next_occurrence {
                ui.label(format!("Next: {}", next.format("%Y-%m-%d %H:%M")));
            }
            
            if let Some(last) = template.last_generated {
                ui.label(format!("Last generated: {}", last.format("%Y-%m-%d %H:%M")));
            }
        });
    }
    
    pub async fn load_templates(&mut self, service: &RecurringService) -> Result<()> {
        self.templates = service.list_active_templates().await?;
        Ok(())
    }
    
    pub async fn create_template(&mut self, service: &RecurringService) -> Result<()> {
        let template = self.editor.build_template();
        let rule = template.recurrence_rule.clone();
        service.create_recurring_template(
            template.title,
            template.description,
            rule.pattern,
            rule.interval,
            if rule.days_of_week.is_empty() { None } else { Some(rule.days_of_week) },
            rule.day_of_month,
            rule.month_of_year,
            rule.time_of_day,
            rule.end_date,
            rule.max_occurrences,
        ).await?;
        Ok(())
    }
    
    pub async fn delete_template(&mut self, service: &RecurringService, id: Uuid) -> Result<()> {
        service.delete_template(id).await?;
        Ok(())
    }
    
    pub async fn toggle_template(&mut self, service: &RecurringService, id: Uuid) -> Result<()> {
        // First check if it's in our active list
        if let Some(template) = self.templates.iter().find(|t| t.id == id) {
            if template.active {
                service.deactivate_template(id).await?;
            } else {
                service.reactivate_template(id).await?;
            }
        } else {
            // If not in active list, it might be inactive, so try to reactivate
            service.reactivate_template(id).await?;
        }
        Ok(())
    }
}