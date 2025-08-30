use crate::domain::goal::{Goal, GoalStatus};
use chrono::Utc;
use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};
use uuid::Uuid;

pub struct GoalView {
    pub new_goal_title: String,
    pub new_goal_description: String,
    pub selected_goal_id: Option<Uuid>,
    pub selected_parent_id: Option<Uuid>,
    pub show_archived: bool,
    pub show_create_form: bool,
    pub editing_goal_id: Option<Uuid>,
    pub edit_title: String,
    pub edit_description: String,
    filter: GoalFilter,
    search_query: String,
}

#[derive(Debug, Clone, PartialEq)]
enum GoalFilter {
    All,
    Active,
    Completed,
    AtRisk,
    NotStarted,
}

impl Default for GoalView {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalView {
    pub fn new() -> Self {
        Self {
            new_goal_title: String::new(),
            new_goal_description: String::new(),
            selected_goal_id: None,
            selected_parent_id: None,
            show_archived: false,
            show_create_form: false,
            editing_goal_id: None,
            edit_title: String::new(),
            edit_description: String::new(),
            filter: GoalFilter::All,
            search_query: String::new(),
        }
    }

    pub fn is_form_valid(&self) -> bool {
        !self.new_goal_title.trim().is_empty()
    }

    pub fn clear_form(&mut self) {
        self.new_goal_title.clear();
        self.new_goal_description.clear();
        self.selected_parent_id = None;
    }

    pub fn show(&mut self, ui: &mut Ui, goals: &mut Vec<Goal>) -> Option<GoalAction> {
        let mut action = None;

        ui.heading("🎯 Goals");

        // Top toolbar
        ui.horizontal(|ui| {
            // Prominent Add Goal button
            if ui.button("➕ New Goal").clicked() {
                self.show_create_form = !self.show_create_form;
            }

            ui.separator();

            // Search bar
            ui.label("🔍");
            if ui.text_edit_singleline(&mut self.search_query).changed() {
                // Trigger search
            }

            ui.separator();

            // Filter buttons
            ui.label("Filter:");
            if ui
                .selectable_label(self.filter == GoalFilter::All, "All")
                .clicked()
            {
                self.filter = GoalFilter::All;
            }
            if ui
                .selectable_label(self.filter == GoalFilter::Active, "Active")
                .clicked()
            {
                self.filter = GoalFilter::Active;
            }
            if ui
                .selectable_label(self.filter == GoalFilter::NotStarted, "Not Started")
                .clicked()
            {
                self.filter = GoalFilter::NotStarted;
            }
            if ui
                .selectable_label(self.filter == GoalFilter::AtRisk, "At Risk")
                .clicked()
            {
                self.filter = GoalFilter::AtRisk;
            }
            if ui
                .selectable_label(self.filter == GoalFilter::Completed, "Completed")
                .clicked()
            {
                self.filter = GoalFilter::Completed;
            }

            ui.separator();

            ui.checkbox(&mut self.show_archived, "Show Archived");
        });

        ui.separator();

        // Create new goal form (now toggleable with the button)
        if self.show_create_form {
            ui.group(|ui| {
                ui.heading("Create New Goal");
                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_goal_title);
                });

                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_goal_description);
                });

                // Parent goal selector
                ui.horizontal(|ui| {
                    ui.label("Parent Goal:");
                    egui::ComboBox::from_label("")
                        .selected_text(
                            self.selected_parent_id
                                .and_then(|id| goals.iter().find(|g| g.id == id))
                                .map(|g| g.title.clone())
                                .unwrap_or_else(|| "None".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(self.selected_parent_id.is_none(), "None")
                                .clicked()
                            {
                                self.selected_parent_id = None;
                            }
                            for goal in goals.iter() {
                                if ui
                                    .selectable_label(
                                        self.selected_parent_id == Some(goal.id),
                                        &goal.title,
                                    )
                                    .clicked()
                                {
                                    self.selected_parent_id = Some(goal.id);
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui.button("✅ Create Goal").clicked() && self.is_form_valid() {
                        action = Some(GoalAction::Create {
                            title: self.new_goal_title.clone(),
                            description: self.new_goal_description.clone(),
                            parent_id: self.selected_parent_id,
                        });
                        self.clear_form();
                        self.show_create_form = false; // Close the form after creation
                    }

                    if ui.button("❌ Cancel").clicked() {
                        self.clear_form();
                        self.show_create_form = false; // Close the form on cancel
                    }
                });
            });
        }

        ui.separator();

        // Goals list
        ScrollArea::vertical().show(ui, |ui| {
            let filtered_goals = self.filter_goals(goals);

            if filtered_goals.is_empty() {
                ui.label("No goals match the current filter");
            } else {
                for goal in filtered_goals {
                    self.show_goal_card(ui, goal, goals, &mut action);
                }
            }
        });

        action
    }

    fn filter_goals<'a>(&self, goals: &'a [Goal]) -> Vec<&'a Goal> {
        goals
            .iter()
            .filter(|goal| {
                // Filter by status
                let status_match = match self.filter {
                    GoalFilter::All => true,
                    GoalFilter::Active => {
                        goal.status == GoalStatus::Active || goal.status == GoalStatus::InProgress
                    }
                    GoalFilter::NotStarted => goal.status == GoalStatus::NotStarted,
                    GoalFilter::AtRisk => goal.status == GoalStatus::AtRisk,
                    GoalFilter::Completed => goal.status == GoalStatus::Completed,
                };

                // Filter by archived status
                let archived_match = self.show_archived || goal.status != GoalStatus::Cancelled;

                // Filter by search query
                let search_match = self.search_query.is_empty()
                    || goal
                        .title
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                    || goal
                        .description
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase());

                status_match && archived_match && search_match
            })
            .collect()
    }

    fn show_goal_card(
        &mut self,
        ui: &mut Ui,
        goal: &Goal,
        all_goals: &[Goal],
        action: &mut Option<GoalAction>,
    ) {
        let is_selected = self.selected_goal_id == Some(goal.id);
        let is_editing = self.editing_goal_id == Some(goal.id);

        ui.group(|ui| {
            // Header with status and title
            ui.horizontal(|ui| {
                // Status icon
                let (status_icon, status_color) = match goal.status {
                    GoalStatus::NotStarted => ("⭕", Color32::GRAY),
                    GoalStatus::Active | GoalStatus::InProgress => {
                        ("🔄", Color32::from_rgb(33, 150, 243))
                    }
                    GoalStatus::OnHold => ("⏸", Color32::from_rgb(255, 193, 7)),
                    GoalStatus::AtRisk => ("⚠️", Color32::from_rgb(255, 87, 34)),
                    GoalStatus::Completed => ("✅", Color32::from_rgb(76, 175, 80)),
                    GoalStatus::Cancelled => ("❌", Color32::from_rgb(244, 67, 54)),
                };
                ui.colored_label(status_color, status_icon);

                // Title (editable if in edit mode)
                if is_editing {
                    ui.text_edit_singleline(&mut self.edit_title);
                } else {
                    let title = if is_selected {
                        RichText::new(&goal.title).strong()
                    } else {
                        RichText::new(&goal.title)
                    };
                    if ui.selectable_label(is_selected, title).clicked() {
                        self.selected_goal_id = Some(goal.id);
                    }
                }

                // Progress bar
                ui.add_space(10.0);
                ui.add(
                    egui::ProgressBar::new(goal.progress / 100.0)
                        .text(format!("{:.0}%", goal.progress)),
                );
            });

            // Description (editable if in edit mode)
            if is_editing {
                ui.text_edit_multiline(&mut self.edit_description);
            } else if !goal.description.is_empty() {
                ui.label(&goal.description);
            }

            // Metadata
            ui.horizontal(|ui| {
                // Target date
                if let Some(target) = goal.target_date {
                    let days_until = (target - Utc::now()).num_days();
                    let date_text = if days_until < 0 {
                        format!("📅 {} days overdue", -days_until)
                    } else if days_until == 0 {
                        "📅 Due today".to_string()
                    } else {
                        format!("📅 {} days remaining", days_until)
                    };

                    let color = if days_until < 0 {
                        Color32::RED
                    } else if days_until <= 7 {
                        Color32::from_rgb(255, 193, 7)
                    } else {
                        ui.visuals().text_color()
                    };

                    ui.colored_label(color, date_text);
                }

                // Parent goal
                if let Some(parent_id) = goal.parent_goal_id
                    && let Some(parent) = all_goals.iter().find(|g| g.id == parent_id)
                {
                    ui.label(format!("📁 {}", parent.title));
                }

                // Estimated hours
                if let Some(hours) = goal.estimated_hours {
                    ui.label(format!("⏱ {}h", hours));
                }
            });

            // Action buttons
            ui.horizontal(|ui| {
                if is_editing {
                    if ui.small_button("✅ Save").clicked() {
                        *action = Some(GoalAction::Update {
                            id: goal.id,
                            title: self.edit_title.clone(),
                            description: self.edit_description.clone(),
                        });
                        self.editing_goal_id = None;
                    }
                    if ui.small_button("❌ Cancel").clicked() {
                        self.editing_goal_id = None;
                    }
                } else {
                    if ui.small_button("✏️ Edit").clicked() {
                        self.editing_goal_id = Some(goal.id);
                        self.edit_title = goal.title.clone();
                        self.edit_description = goal.description.clone();
                    }

                    // Status change buttons
                    match goal.status {
                        GoalStatus::NotStarted => {
                            if ui.small_button("▶️ Start").clicked() {
                                *action = Some(GoalAction::ChangeStatus {
                                    id: goal.id,
                                    status: GoalStatus::Active,
                                });
                            }
                        }
                        GoalStatus::Active | GoalStatus::InProgress => {
                            if ui.small_button("⏸ Hold").clicked() {
                                *action = Some(GoalAction::ChangeStatus {
                                    id: goal.id,
                                    status: GoalStatus::OnHold,
                                });
                            }
                            if ui.small_button("✅ Complete").clicked() {
                                *action = Some(GoalAction::ChangeStatus {
                                    id: goal.id,
                                    status: GoalStatus::Completed,
                                });
                            }
                        }
                        GoalStatus::OnHold => {
                            if ui.small_button("▶️ Resume").clicked() {
                                *action = Some(GoalAction::ChangeStatus {
                                    id: goal.id,
                                    status: GoalStatus::Active,
                                });
                            }
                        }
                        _ => {}
                    }

                    if ui.small_button("🗑 Delete").clicked() {
                        *action = Some(GoalAction::Delete { id: goal.id });
                    }
                }
            });
        });
    }
}

#[derive(Debug, Clone)]
pub enum GoalAction {
    Create {
        title: String,
        description: String,
        parent_id: Option<Uuid>,
    },
    Update {
        id: Uuid,
        title: String,
        description: String,
    },
    Delete {
        id: Uuid,
    },
    ChangeStatus {
        id: Uuid,
        status: GoalStatus,
    },
}
