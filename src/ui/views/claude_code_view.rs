use crate::domain::claude_code::{ClaudeCodeConfig, ClaudeCodeSession, SessionStatus};
use crate::repository::Repository;
use crate::services::ClaudeCodeService;
use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};
use uuid::Uuid;

pub struct ClaudeCodeView {
    sessions: Vec<ClaudeCodeSession>,
    selected_session: Option<Uuid>,
    config: Option<ClaudeCodeConfig>,
    service: Option<ClaudeCodeService>,
    log_filter: String,
    show_config_panel: bool,
    temp_config: ClaudeCodeConfig,
}

impl ClaudeCodeView {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_session: None,
            config: None,
            service: None,
            log_filter: String::new(),
            show_config_panel: false,
            temp_config: ClaudeCodeConfig::new("".to_string(), "".to_string()),
        }
    }

    pub async fn init(&mut self, repository: &Repository) {
        // Load configuration
        if let Ok(Some(config)) = repository.claude_code.get_config().await {
            self.temp_config = config.clone();
            self.config = Some(config);
        }

        // Initialize service
        self.service = Some(ClaudeCodeService::new(repository.claude_code.clone()));

        // Load all sessions
        self.refresh_sessions(repository).await;
    }

    pub async fn refresh_sessions(&mut self, repository: &Repository) {
        // For now, get active sessions - in production, you'd want pagination
        if let Ok(sessions) = repository.claude_code.get_active_sessions().await {
            self.sessions = sessions;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, repository: &Repository) {
        ui.heading("Claude Code Sessions");

        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                // Use tokio runtime if available, otherwise skip
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let repo_clone = repository.clone();
                    let sessions_future =
                        async move { repo_clone.claude_code.get_active_sessions().await };

                    // Block on the future to get results synchronously
                    if let Ok(sessions) = handle.block_on(sessions_future) {
                        self.sessions = sessions;
                    }
                } else {
                    // No runtime available - show message
                    ui.label("(Runtime not available for refresh)");
                }
            }

            ui.separator();

            if ui
                .button(if self.show_config_panel {
                    "Hide Config"
                } else {
                    "Show Config"
                })
                .clicked()
            {
                self.show_config_panel = !self.show_config_panel;
            }

            ui.separator();

            ui.label("Filter logs:");
            ui.text_edit_singleline(&mut self.log_filter);
        });

        ui.separator();

        // Configuration panel
        if self.show_config_panel {
            self.show_config_editor(ui);
            ui.separator();
        }

        // Main content area with two columns
        ui.columns(2, |columns| {
            // Left column: Session list
            columns[0].heading("Sessions");

            ScrollArea::vertical()
                .max_height(600.0)
                .show(&mut columns[0], |ui| {
                    if self.sessions.is_empty() {
                        ui.label("No active sessions");
                    } else {
                        for session in &self.sessions {
                            let is_selected = self.selected_session == Some(session.id);

                            ui.group(|ui| {
                                ui.set_width(ui.available_width());

                                let response = ui.selectable_label(
                                    is_selected,
                                    format!("Session: {}", &session.id.to_string()[..8]),
                                );

                                if response.clicked() {
                                    self.selected_session = Some(session.id);
                                }

                                ui.horizontal(|ui| {
                                    let (status_text, status_color) = match session.status {
                                        SessionStatus::Pending => ("Pending", Color32::GRAY),
                                        SessionStatus::Initializing => {
                                            ("Initializing", Color32::YELLOW)
                                        }
                                        SessionStatus::Working => {
                                            ("Working", Color32::from_rgb(255, 165, 0))
                                        }
                                        SessionStatus::CreatingPR => {
                                            ("Creating PR", Color32::from_rgb(0, 150, 255))
                                        }
                                        SessionStatus::Completed => ("Completed", Color32::GREEN),
                                        SessionStatus::Failed => ("Failed", Color32::RED),
                                        SessionStatus::Cancelled => {
                                            ("Cancelled", Color32::DARK_GRAY)
                                        }
                                    };

                                    ui.colored_label(status_color, status_text);

                                    if session.status.is_active() {
                                        ui.spinner();
                                    }
                                });

                                if let Some(branch) = &session.branch_name {
                                    ui.label(format!("Branch: {}", branch));
                                }

                                if let Some(pr_url) = &session.pr_url {
                                    ui.hyperlink_to("View PR", pr_url);
                                }

                                ui.label(format!(
                                    "Started: {}",
                                    session.started_at.format("%Y-%m-%d %H:%M:%S")
                                ));

                                if let Some(duration) = session.duration() {
                                    ui.label(format!("Duration: {}m", duration.num_minutes()));
                                }

                                // Action buttons
                                ui.horizontal(|ui| {
                                    if session.status.is_active()
                                        && ui.button("Cancel").clicked()
                                        && let Some(_service) = &mut self.service
                                    {
                                        let _session_id = session.id;
                                        tokio::spawn(async move {
                                            // Cancel would be handled by the service
                                        });
                                    }

                                    if session.status.is_terminal()
                                        && session.pr_url.is_none()
                                        && ui.button("Retry").clicked()
                                    {
                                        // Retry logic would go here
                                    }
                                });
                            });

                            ui.add_space(5.0);
                        }
                    }
                });

            // Right column: Session details
            columns[1].heading("Session Details");

            if let Some(selected_id) = self.selected_session {
                if let Some(session) = self.sessions.iter().find(|s| s.id == selected_id) {
                    self.show_session_details(&mut columns[1], session);
                }
            } else {
                columns[1].label("Select a session to view details");
            }
        });
    }

    fn show_session_details(&self, ui: &mut Ui, session: &ClaudeCodeSession) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());

            ui.label(RichText::new("Session Information").strong());

            ui.horizontal(|ui| {
                ui.label("ID:");
                ui.monospace(session.id.to_string());
            });

            ui.horizontal(|ui| {
                ui.label("Task ID:");
                ui.monospace(session.task_id.to_string());
            });

            if let Some(branch) = &session.branch_name {
                ui.horizontal(|ui| {
                    ui.label("Branch:");
                    ui.monospace(branch);
                });
            }

            if let Some(pr_url) = &session.pr_url {
                ui.horizontal(|ui| {
                    ui.label("Pull Request:");
                    ui.hyperlink(pr_url);
                });

                if let Some(pr_num) = session.pr_number {
                    ui.label(format!("PR #{}", pr_num));
                }
            }

            if let Some(error) = &session.error_message {
                ui.separator();
                ui.colored_label(Color32::RED, "Error:");
                ui.label(error);
            }
        });

        ui.add_space(10.0);

        // Session log
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Session Log").strong());

            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                let log_text = if self.log_filter.is_empty() {
                    session.session_log.clone()
                } else {
                    session
                        .session_log
                        .lines()
                        .filter(|line| line.contains(&self.log_filter))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                if log_text.is_empty() {
                    ui.label("No log entries");
                } else {
                    ui.monospace(log_text);
                }
            });
        });
    }

    fn show_config_editor(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Claude Code Configuration").strong());

            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("GitHub Owner:");
                    ui.text_edit_singleline(&mut self.temp_config.github_owner);
                    ui.end_row();

                    ui.label("GitHub Repo:");
                    ui.text_edit_singleline(&mut self.temp_config.github_repo);
                    ui.end_row();

                    ui.label("Default Branch:");
                    ui.text_edit_singleline(&mut self.temp_config.default_base_branch);
                    ui.end_row();

                    ui.label("Working Directory:");
                    let work_dir = self.temp_config.working_directory.as_deref().unwrap_or("");
                    let mut work_dir_str = work_dir.to_string();
                    ui.text_edit_singleline(&mut work_dir_str);
                    self.temp_config.working_directory = if work_dir_str.is_empty() {
                        None
                    } else {
                        Some(work_dir_str)
                    };
                    ui.end_row();

                    ui.label("Claude Model:");
                    ui.text_edit_singleline(&mut self.temp_config.claude_model);
                    ui.end_row();

                    ui.label("Max Duration (min):");
                    ui.add(
                        egui::DragValue::new(&mut self.temp_config.max_session_duration_minutes)
                            .speed(1)
                            .clamp_range(5..=240),
                    );
                    ui.end_row();

                    ui.label("Auto Create PR:");
                    ui.checkbox(&mut self.temp_config.auto_create_pr, "");
                    ui.end_row();

                    ui.label("GitHub Token:");
                    let token = self.temp_config.github_token.as_deref().unwrap_or("");
                    let mut token_str = if token.is_empty() {
                        String::new()
                    } else {
                        "***hidden***".to_string()
                    };
                    if ui.text_edit_singleline(&mut token_str).changed()
                        && token_str != "***hidden***"
                    {
                        self.temp_config.github_token = Some(token_str);
                    }
                    ui.end_row();

                    ui.label("Claude API Key:");
                    let api_key = self.temp_config.claude_api_key.as_deref().unwrap_or("");
                    let mut api_key_str = if api_key.is_empty() {
                        String::new()
                    } else {
                        "***hidden***".to_string()
                    };
                    if ui.text_edit_singleline(&mut api_key_str).changed()
                        && api_key_str != "***hidden***"
                    {
                        self.temp_config.claude_api_key = Some(api_key_str);
                    }
                    ui.end_row();
                });

            ui.horizontal(|ui| {
                if ui.button("Save Configuration").clicked() {
                    if let Err(e) = self.temp_config.validate() {
                        // Show error - in production, you'd show a toast or modal
                        ui.colored_label(Color32::RED, format!("Validation error: {}", e));
                    } else {
                        self.config = Some(self.temp_config.clone());
                        // Save to database would happen here via async task
                    }
                }

                if ui.button("Reset").clicked()
                    && let Some(config) = &self.config
                {
                    self.temp_config = config.clone();
                }
            });
        });
    }
}

impl Default for ClaudeCodeView {
    fn default() -> Self {
        Self::new()
    }
}
