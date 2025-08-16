use crate::domain::task_config::{
    TaskConfiguration, MetadataFieldConfig, FieldType, FieldOption, 
    StateDefinition, StateTransition, ValidationRule, TransitionCondition, 
    TransitionEffect, AutoAction
};
use crate::services::TaskConfigService;
use eframe::egui::{self, Ui, Context, Vec2, Color32, RichText};
use std::sync::Arc;
use uuid::Uuid;

pub struct MetadataConfigView {
    configs: Vec<TaskConfiguration>,
    selected_config_id: Option<Uuid>,
    
    show_new_config_dialog: bool,
    new_config_name: String,
    new_config_description: String,
    
    show_field_editor: bool,
    editing_field: Option<MetadataFieldConfig>,
    field_name: String,
    field_display_name: String,
    field_type: FieldType,
    field_required: bool,
    field_default: String,
    field_help_text: String,
    field_options: Vec<FieldOption>,
    field_show_in_list: bool,
    field_show_in_card: bool,
    field_sortable: bool,
    field_searchable: bool,
    
    show_state_editor: bool,
    editing_state: Option<StateDefinition>,
    state_name: String,
    state_display_name: String,
    state_color: String,
    state_description: String,
    state_is_final: bool,
    
    show_transition_editor: bool,
    transition_from: String,
    transition_to: String,
    transition_action: String,
    
    active_tab: ConfigTab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfigTab {
    Overview,
    MetadataFields,
    StateMachine,
    Presets,
}

impl MetadataConfigView {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            selected_config_id: None,
            
            show_new_config_dialog: false,
            new_config_name: String::new(),
            new_config_description: String::new(),
            
            show_field_editor: false,
            editing_field: None,
            field_name: String::new(),
            field_display_name: String::new(),
            field_type: FieldType::Text,
            field_required: false,
            field_default: String::new(),
            field_help_text: String::new(),
            field_options: Vec::new(),
            field_show_in_list: true,
            field_show_in_card: true,
            field_sortable: false,
            field_searchable: false,
            
            show_state_editor: false,
            editing_state: None,
            state_name: String::new(),
            state_display_name: String::new(),
            state_color: "#808080".to_string(),
            state_description: String::new(),
            state_is_final: false,
            
            show_transition_editor: false,
            transition_from: String::new(),
            transition_to: String::new(),
            transition_action: String::new(),
            
            active_tab: ConfigTab::Overview,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, service: Option<Arc<TaskConfigService>>) {
        ui.heading("Task Metadata Configuration");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ New Configuration").clicked() {
                self.show_new_config_dialog = true;
            }
            
            if ui.button("📥 Import Preset").clicked() {
                self.active_tab = ConfigTab::Presets;
            }
            
            ui.separator();
            
            ui.selectable_value(&mut self.active_tab, ConfigTab::Overview, "Overview");
            ui.selectable_value(&mut self.active_tab, ConfigTab::MetadataFields, "Metadata Fields");
            ui.selectable_value(&mut self.active_tab, ConfigTab::StateMachine, "State Machine");
            ui.selectable_value(&mut self.active_tab, ConfigTab::Presets, "Presets");
        });

        ui.separator();

        egui::SidePanel::left("config_list")
            .default_width(200.0)
            .show_inside(ui, |ui| {
                self.show_config_list(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(config_id) = self.selected_config_id {
                let config_idx = self.configs.iter().position(|c| c.id == config_id);
                if let Some(idx) = config_idx {
                    match self.active_tab {
                        ConfigTab::Overview => {
                            let config = &self.configs[idx];
                            self.show_overview(ui, config);
                        }
                        ConfigTab::MetadataFields => {
                            let config = self.configs[idx].clone();
                            self.show_metadata_fields(ui, &config);
                        }
                        ConfigTab::StateMachine => {
                            let config = self.configs[idx].clone();
                            self.show_state_machine(ui, &config);
                        }
                        ConfigTab::Presets => self.show_presets(ui),
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a configuration or create a new one");
                });
            }
        });

        self.show_dialogs(ui.ctx());
    }

    fn show_config_list(&mut self, ui: &mut Ui) {
        ui.heading("Configurations");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for config in &self.configs {
                let is_selected = self.selected_config_id == Some(config.id);
                
                if ui.selectable_label(is_selected, &config.name).clicked() {
                    self.selected_config_id = Some(config.id);
                }
            }
        });
    }

    fn show_overview(&self, ui: &mut Ui, config: &TaskConfiguration) {
        ui.heading(&config.name);
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.label(&config.description);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Created:");
            ui.label(config.created_at.format("%Y-%m-%d %H:%M").to_string());
        });

        ui.horizontal(|ui| {
            ui.label("Updated:");
            ui.label(config.updated_at.format("%Y-%m-%d %H:%M").to_string());
        });

        ui.separator();

        ui.heading("Statistics");
        
        ui.horizontal(|ui| {
            ui.label(format!("Metadata Fields: {}", config.metadata_schema.fields.len()));
        });

        ui.horizontal(|ui| {
            ui.label(format!("States: {}", config.state_machine.states.len()));
        });

        ui.horizontal(|ui| {
            ui.label(format!("Transitions: {}", config.state_machine.transitions.len()));
        });
    }

    fn show_metadata_fields(&mut self, ui: &mut Ui, config: &TaskConfiguration) {
        ui.heading("Metadata Fields");
        
        ui.horizontal(|ui| {
            if ui.button("➕ Add Field").clicked() {
                self.show_field_editor = true;
                self.editing_field = None;
                self.reset_field_editor();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, field) in &config.metadata_schema.fields {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&field.display_name).strong());
                        ui.label(format!("({})", name));
                        
                        if field.required {
                            ui.colored_label(Color32::RED, "Required");
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✏️").clicked() {
                                self.show_field_editor = true;
                                self.editing_field = Some(field.clone());
                                self.load_field_to_editor(field);
                            }
                            
                            if ui.button("🗑️").clicked() {
                                // TODO: Remove field
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        ui.label(format!("{:?}", field.field_type));
                    });

                    if !field.help_text.is_empty() {
                        ui.label(&field.help_text);
                    }

                    if let Some(default) = &field.default_value {
                        ui.horizontal(|ui| {
                            ui.label("Default:");
                            ui.label(default);
                        });
                    }

                    if !field.options.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Options:");
                            for option in &field.options {
                                ui.label(&option.label);
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        if field.show_in_list {
                            ui.label("📋 List");
                        }
                        if field.show_in_card {
                            ui.label("🎴 Card");
                        }
                        if field.sortable {
                            ui.label("🔤 Sortable");
                        }
                        if field.searchable {
                            ui.label("🔍 Searchable");
                        }
                    });
                });
            }
        });
    }

    fn show_state_machine(&mut self, ui: &mut Ui, config: &TaskConfiguration) {
        ui.heading("State Machine");
        
        ui.horizontal(|ui| {
            if ui.button("➕ Add State").clicked() {
                self.show_state_editor = true;
                self.editing_state = None;
                self.reset_state_editor();
            }
            
            if ui.button("➕ Add Transition").clicked() {
                self.show_transition_editor = true;
                self.reset_transition_editor();
            }
        });

        ui.separator();

        ui.heading("States");
        
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (name, state) in &config.state_machine.states {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let color = parse_color(&state.color);
                            ui.colored_label(color, "●");
                            ui.label(RichText::new(&state.display_name).strong());
                            
                            if state.is_final {
                                ui.colored_label(Color32::GREEN, "Final");
                            }
                            
                            if name == &config.state_machine.initial_state {
                                ui.colored_label(Color32::BLUE, "Initial");
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("✏️").clicked() {
                                    self.show_state_editor = true;
                                    self.editing_state = Some(state.clone());
                                    self.load_state_to_editor(state);
                                }
                                
                                if ui.button("🗑️").clicked() {
                                    // TODO: Remove state
                                }
                            });
                        });
                        
                        ui.label(&state.description);
                    });
                }
            });

        ui.separator();
        ui.heading("Transitions");
        
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for transition in &config.state_machine.transitions {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&transition.from_state);
                            ui.label("→");
                            ui.label(&transition.to_state);
                            ui.label(format!("[{}]", transition.action_name));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️").clicked() {
                                    // TODO: Remove transition
                                }
                            });
                        });
                        
                        if !transition.conditions.is_empty() {
                            ui.label(format!("Conditions: {}", transition.conditions.len()));
                        }
                        
                        if !transition.effects.is_empty() {
                            ui.label(format!("Effects: {}", transition.effects.len()));
                        }
                    });
                }
            });
    }

    fn show_presets(&self, ui: &mut Ui) {
        ui.heading("Configuration Presets");
        ui.separator();

        ui.label("Import a preset configuration to quickly get started:");
        
        ui.group(|ui| {
            ui.heading("Software Development");
            ui.label("Includes fields for story points, sprints, PR URLs, and a review state.");
            if ui.button("Import").clicked() {
                // TODO: Import software development preset
            }
        });

        ui.group(|ui| {
            ui.heading("Marketing Campaign");
            ui.label("Includes fields for campaign type, budget, target audience, and approval workflow.");
            if ui.button("Import").clicked() {
                // TODO: Import marketing preset
            }
        });

        ui.group(|ui| {
            ui.heading("Bug Tracking");
            ui.label("Includes severity, affected version, steps to reproduce, and triage workflow.");
            if ui.button("Import").clicked() {
                // TODO: Import bug tracking preset
            }
        });

        ui.group(|ui| {
            ui.heading("Content Production");
            ui.label("Includes content type, word count, SEO keywords, and editorial workflow.");
            if ui.button("Import").clicked() {
                // TODO: Import content production preset
            }
        });
    }

    fn show_dialogs(&mut self, ctx: &Context) {
        if self.show_new_config_dialog {
            egui::Window::new("New Configuration")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_config_name);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_multiline(&mut self.new_config_description);
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            // TODO: Create new configuration
                            self.show_new_config_dialog = false;
                            self.new_config_name.clear();
                            self.new_config_description.clear();
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_new_config_dialog = false;
                            self.new_config_name.clear();
                            self.new_config_description.clear();
                        }
                    });
                });
        }

        if self.show_field_editor {
            self.show_field_editor_dialog(ctx);
        }

        if self.show_state_editor {
            self.show_state_editor_dialog(ctx);
        }

        if self.show_transition_editor {
            self.show_transition_editor_dialog(ctx);
        }
    }

    fn show_field_editor_dialog(&mut self, ctx: &Context) {
        let title = if self.editing_field.is_some() {
            "Edit Field"
        } else {
            "Add Field"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Field Name:");
                    ui.text_edit_singleline(&mut self.field_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Display Name:");
                    ui.text_edit_singleline(&mut self.field_display_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.field_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.field_type, FieldType::Text, "Text");
                            ui.selectable_value(&mut self.field_type, FieldType::LongText, "Long Text");
                            ui.selectable_value(&mut self.field_type, FieldType::Number, "Number");
                            ui.selectable_value(&mut self.field_type, FieldType::Decimal, "Decimal");
                            ui.selectable_value(&mut self.field_type, FieldType::Date, "Date");
                            ui.selectable_value(&mut self.field_type, FieldType::DateTime, "Date Time");
                            ui.selectable_value(&mut self.field_type, FieldType::Select, "Select");
                            ui.selectable_value(&mut self.field_type, FieldType::MultiSelect, "Multi Select");
                            ui.selectable_value(&mut self.field_type, FieldType::Boolean, "Boolean");
                            ui.selectable_value(&mut self.field_type, FieldType::Url, "URL");
                            ui.selectable_value(&mut self.field_type, FieldType::Email, "Email");
                            ui.selectable_value(&mut self.field_type, FieldType::Phone, "Phone");
                            ui.selectable_value(&mut self.field_type, FieldType::Currency, "Currency");
                            ui.selectable_value(&mut self.field_type, FieldType::Percentage, "Percentage");
                        });
                });

                ui.checkbox(&mut self.field_required, "Required");

                ui.horizontal(|ui| {
                    ui.label("Default Value:");
                    ui.text_edit_singleline(&mut self.field_default);
                });

                ui.horizontal(|ui| {
                    ui.label("Help Text:");
                    ui.text_edit_multiline(&mut self.field_help_text);
                });

                if self.field_type == FieldType::Select || self.field_type == FieldType::MultiSelect {
                    ui.separator();
                    ui.label("Options:");
                    
                    for i in 0..self.field_options.len() {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.field_options[i].value);
                            ui.text_edit_singleline(&mut self.field_options[i].label);
                            if ui.button("🗑️").clicked() {
                                self.field_options.remove(i);
                            }
                        });
                    }
                    
                    if ui.button("➕ Add Option").clicked() {
                        self.field_options.push(FieldOption {
                            value: String::new(),
                            label: String::new(),
                            color: None,
                            icon: None,
                        });
                    }
                }

                ui.separator();
                ui.label("Display Options:");
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.field_show_in_list, "Show in List");
                    ui.checkbox(&mut self.field_show_in_card, "Show in Card");
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.field_sortable, "Sortable");
                    ui.checkbox(&mut self.field_searchable, "Searchable");
                });

                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        // TODO: Save field
                        self.show_field_editor = false;
                        self.reset_field_editor();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_field_editor = false;
                        self.reset_field_editor();
                    }
                });
            });
    }

    fn show_state_editor_dialog(&mut self, ctx: &Context) {
        let title = if self.editing_state.is_some() {
            "Edit State"
        } else {
            "Add State"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("State Name:");
                    ui.text_edit_singleline(&mut self.state_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Display Name:");
                    ui.text_edit_singleline(&mut self.state_display_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.text_edit_singleline(&mut self.state_color);
                    let color = parse_color(&self.state_color);
                    ui.colored_label(color, "●");
                });

                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.state_description);
                });

                ui.checkbox(&mut self.state_is_final, "Final State");

                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        // TODO: Save state
                        self.show_state_editor = false;
                        self.reset_state_editor();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_state_editor = false;
                        self.reset_state_editor();
                    }
                });
            });
    }

    fn show_transition_editor_dialog(&mut self, ctx: &Context) {
        egui::Window::new("Add Transition")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("From State:");
                    ui.text_edit_singleline(&mut self.transition_from);
                });

                ui.horizontal(|ui| {
                    ui.label("To State:");
                    ui.text_edit_singleline(&mut self.transition_to);
                });

                ui.horizontal(|ui| {
                    ui.label("Action Name:");
                    ui.text_edit_singleline(&mut self.transition_action);
                });

                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        // TODO: Save transition
                        self.show_transition_editor = false;
                        self.reset_transition_editor();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_transition_editor = false;
                        self.reset_transition_editor();
                    }
                });
            });
    }

    fn reset_field_editor(&mut self) {
        self.field_name.clear();
        self.field_display_name.clear();
        self.field_type = FieldType::Text;
        self.field_required = false;
        self.field_default.clear();
        self.field_help_text.clear();
        self.field_options.clear();
        self.field_show_in_list = true;
        self.field_show_in_card = true;
        self.field_sortable = false;
        self.field_searchable = false;
    }

    fn load_field_to_editor(&mut self, field: &MetadataFieldConfig) {
        self.field_name = field.name.clone();
        self.field_display_name = field.display_name.clone();
        self.field_type = field.field_type;
        self.field_required = field.required;
        self.field_default = field.default_value.clone().unwrap_or_default();
        self.field_help_text = field.help_text.clone();
        self.field_options = field.options.clone();
        self.field_show_in_list = field.show_in_list;
        self.field_show_in_card = field.show_in_card;
        self.field_sortable = field.sortable;
        self.field_searchable = field.searchable;
    }

    fn reset_state_editor(&mut self) {
        self.state_name.clear();
        self.state_display_name.clear();
        self.state_color = "#808080".to_string();
        self.state_description.clear();
        self.state_is_final = false;
    }

    fn load_state_to_editor(&mut self, state: &StateDefinition) {
        self.state_name = state.name.clone();
        self.state_display_name = state.display_name.clone();
        self.state_color = state.color.clone();
        self.state_description = state.description.clone();
        self.state_is_final = state.is_final;
    }

    fn reset_transition_editor(&mut self) {
        self.transition_from.clear();
        self.transition_to.clear();
        self.transition_action.clear();
    }
}

fn parse_color(hex: &str) -> Color32 {
    if hex.len() == 7 && hex.starts_with('#') {
        if let Ok(r) = u8::from_str_radix(&hex[1..3], 16) {
            if let Ok(g) = u8::from_str_radix(&hex[3..5], 16) {
                if let Ok(b) = u8::from_str_radix(&hex[5..7], 16) {
                    return Color32::from_rgb(r, g, b);
                }
            }
        }
    }
    Color32::GRAY
}