use crate::domain::resource::Resource;
use eframe::egui::{self, Ui, Color32};
use uuid::Uuid;

pub struct ResourceView {
    pub selected_resource: Option<Uuid>,
    pub new_resource_name: String,
    pub new_resource_role: String,
    pub new_resource_hours: f32,
    pub show_create_dialog: bool,
    pub show_edit_dialog: bool,
    pub edit_resource_name: String,
    pub edit_resource_role: String,
    pub edit_resource_hours: f32,
    pub edit_resource_email: String,
    pub edit_resource_skills: Vec<String>,
    pub new_skill: String,
    pub filter_text: String,
}

impl Default for ResourceView {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceView {
    pub fn new() -> Self {
        Self {
            selected_resource: None,
            new_resource_name: String::new(),
            new_resource_role: String::new(),
            new_resource_hours: 40.0,
            show_create_dialog: false,
            show_edit_dialog: false,
            edit_resource_name: String::new(),
            edit_resource_role: String::new(),
            edit_resource_hours: 40.0,
            edit_resource_email: String::new(),
            edit_resource_skills: Vec::new(),
            new_skill: String::new(),
            filter_text: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, resources: &mut Vec<Resource>) {
        ui.heading("Resource Management");
        
        ui.horizontal(|ui| {
            if ui.button("➕ Add Resource").clicked() {
                self.show_create_dialog = true;
                self.new_resource_name.clear();
                self.new_resource_role.clear();
                self.new_resource_hours = 40.0;
            }
            
            ui.separator();
            
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
        });
        
        ui.separator();
        
        // Create new resource dialog
        if self.show_create_dialog {
            egui::Window::new("Add New Resource")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("new_resource_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_resource_name);
                            ui.end_row();
                            
                            ui.label("Role:");
                            ui.text_edit_singleline(&mut self.new_resource_role);
                            ui.end_row();
                            
                            ui.label("Weekly Hours:");
                            ui.add(egui::Slider::new(&mut self.new_resource_hours, 1.0..=60.0));
                            ui.end_row();
                        });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked()
                            && !self.new_resource_name.trim().is_empty() {
                                let resource = Resource::new(
                                    self.new_resource_name.clone(),
                                    self.new_resource_role.clone(),
                                    self.new_resource_hours,
                                );
                                resources.push(resource);
                                self.show_create_dialog = false;
                            }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_create_dialog = false;
                        }
                    });
                });
        }
        
        // Edit resource dialog
        if self.show_edit_dialog {
            if let Some(resource_id) = self.selected_resource {
                egui::Window::new("Edit Resource")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        egui::Grid::new("edit_resource_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.edit_resource_name);
                                ui.end_row();
                                
                                ui.label("Role:");
                                ui.text_edit_singleline(&mut self.edit_resource_role);
                                ui.end_row();
                                
                                ui.label("Email:");
                                ui.text_edit_singleline(&mut self.edit_resource_email);
                                ui.end_row();
                                
                                ui.label("Weekly Hours:");
                                ui.add(egui::Slider::new(&mut self.edit_resource_hours, 1.0..=60.0));
                                ui.end_row();
                                
                                ui.label("Skills:");
                                ui.vertical(|ui| {
                                    // Display existing skills with remove buttons
                                    let mut skills_to_remove = Vec::new();
                                    for (idx, skill) in self.edit_resource_skills.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(skill);
                                            if ui.small_button("❌").clicked() {
                                                skills_to_remove.push(idx);
                                            }
                                        });
                                    }
                                    // Remove skills marked for deletion
                                    for idx in skills_to_remove.into_iter().rev() {
                                        self.edit_resource_skills.remove(idx);
                                    }
                                    
                                    // Add new skill input
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut self.new_skill);
                                        if ui.button("Add Skill").clicked() && !self.new_skill.trim().is_empty() {
                                            self.edit_resource_skills.push(self.new_skill.clone());
                                            self.new_skill.clear();
                                        }
                                    });
                                });
                                ui.end_row();
                            });
                        
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked()
                                && !self.edit_resource_name.trim().is_empty() {
                                    // Find and update the resource
                                    if let Some(resource) = resources.iter_mut().find(|r| r.id == resource_id) {
                                        resource.name = self.edit_resource_name.clone();
                                        resource.role = self.edit_resource_role.clone();
                                        resource.weekly_hours = self.edit_resource_hours;
                                        resource.email = if self.edit_resource_email.trim().is_empty() {
                                            None
                                        } else {
                                            Some(self.edit_resource_email.clone())
                                        };
                                        resource.skills.clear();
                                        for skill in &self.edit_resource_skills {
                                            resource.skills.insert(skill.clone());
                                        }
                                        resource.updated_at = chrono::Utc::now();
                                    }
                                    self.show_edit_dialog = false;
                                    self.selected_resource = None;
                                }
                            
                            if ui.button("Cancel").clicked() {
                                self.show_edit_dialog = false;
                                self.selected_resource = None;
                            }
                            
                            ui.separator();
                            
                            // Add delete button with confirmation
                            if ui.button("🗑 Delete").clicked() {
                                // Remove the resource
                                resources.retain(|r| r.id != resource_id);
                                self.show_edit_dialog = false;
                                self.selected_resource = None;
                            }
                        });
                    });
            }
        }
        
        // Resource list
        let filtered_resources: Vec<&Resource> = resources.iter()
            .filter(|r| {
                if self.filter_text.is_empty() {
                    true
                } else {
                    r.name.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                    r.role.to_lowercase().contains(&self.filter_text.to_lowercase())
                }
            })
            .collect();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for resource in filtered_resources {
                    self.show_resource_card(ui, resource);
                    ui.separator();
                }
                
                if resources.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.label("No resources found");
                        ui.label("Click 'Add Resource' to create your first resource");
                    });
                }
            });
    }
    
    fn show_resource_card(&mut self, ui: &mut Ui, resource: &Resource) {
        egui::Frame::none()
            .fill(Color32::from_gray(250))
            .stroke(egui::Stroke::new(1.0, Color32::from_gray(200)))
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(&resource.name);
                            ui.label(format!("({})", resource.role));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("📅");
                            ui.label(format!("{} hours/week", resource.weekly_hours));
                            
                            ui.separator();
                            
                            // Utilization indicator
                            let utilization = resource.utilization_percentage();
                            let color = if utilization > 100.0 {
                                Color32::RED
                            } else if utilization > 80.0 {
                                Color32::from_rgb(255, 165, 0) // Orange
                            } else {
                                Color32::from_rgb(76, 175, 80) // Green
                            };
                            
                            ui.colored_label(color, format!("{}% utilized", utilization.round()));
                            
                            if resource.is_overloaded() {
                                ui.colored_label(Color32::RED, "⚠ Overloaded");
                            }
                        });
                        
                        // Skills
                        if !resource.skills.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("🔧 Skills:");
                                for skill in &resource.skills {
                                    let _ = ui.small_button(skill);
                                }
                            });
                        }
                        
                        // Email if available
                        if let Some(email) = &resource.email {
                            ui.horizontal(|ui| {
                                ui.label("📧");
                                ui.label(email);
                            });
                        }
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✏").clicked() {
                            self.selected_resource = Some(resource.id);
                            self.show_edit_dialog = true;
                            // Populate edit fields with current resource data
                            self.edit_resource_name = resource.name.clone();
                            self.edit_resource_role = resource.role.clone();
                            self.edit_resource_hours = resource.weekly_hours;
                            self.edit_resource_email = resource.email.clone().unwrap_or_default();
                            self.edit_resource_skills = resource.skills.iter().cloned().collect();
                            self.new_skill.clear();
                        }
                        
                        ui.label(format!("{:.1}h available", resource.available_hours()));
                    });
                });
            });
    }
    
    pub fn show_resource_utilization_chart(&self, ui: &mut Ui, resources: &[Resource]) {
        ui.heading("Resource Utilization");
        
        for resource in resources {
            ui.horizontal(|ui| {
                ui.label(&resource.name);
                
                let utilization = resource.utilization_percentage();
                let progress = (utilization / 100.0).min(1.0);
                
                let color = if utilization > 100.0 {
                    Color32::RED
                } else if utilization > 80.0 {
                    Color32::from_rgb(255, 165, 0)
                } else {
                    Color32::from_rgb(76, 175, 80)
                };
                
                let progress_bar = egui::ProgressBar::new(progress)
                    .fill(color)
                    .text(format!("{}%", utilization.round()));
                    
                ui.add(progress_bar);
            });
        }
    }
}