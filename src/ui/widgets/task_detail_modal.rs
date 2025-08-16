use crate::domain::{task::Task, comment::{Comment, EntityType}, resource::Resource};
use crate::repository::comment_repository::CommentRepository;
use eframe::egui::{self, Ui, Window, ScrollArea, Vec2, Color32};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

pub struct TaskDetailModal {
    pub visible: bool,
    pub task: Option<Task>,
    pub comments: Vec<Comment>,
    pub new_comment_text: String,
    pub editing_comment_id: Option<Uuid>,
    pub edit_comment_text: String,
    pub comment_repository: Option<Arc<CommentRepository>>,
    pub show_edit_mode: bool,
}

impl Default for TaskDetailModal {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskDetailModal {
    pub fn new() -> Self {
        Self {
            visible: false,
            task: None,
            comments: Vec::new(),
            new_comment_text: String::new(),
            editing_comment_id: None,
            edit_comment_text: String::new(),
            comment_repository: None,
            show_edit_mode: false,
        }
    }

    pub fn open(&mut self, task: Task, comments: Vec<Comment>) {
        self.visible = true;
        self.task = Some(task);
        self.comments = comments;
        self.new_comment_text.clear();
        self.editing_comment_id = None;
        self.show_edit_mode = false;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.task = None;
        self.comments.clear();
        self.new_comment_text.clear();
        self.editing_comment_id = None;
        self.show_edit_mode = false;
    }

    pub fn show(&mut self, ctx: &egui::Context, resources: &[Resource]) -> Option<TaskAction> {
        if !self.visible {
            return None;
        }

        let mut action = None;
        let mut should_close = false;

        // Clone task for display to avoid borrow issues
        let task_clone = self.task.clone();
        
        if let Some(task) = task_clone {
            let window_title = format!("Task: {}", task.title);
            
            Window::new(window_title)
                .id(egui::Id::new("task_detail_modal"))
                .default_size(Vec2::new(800.0, 600.0))
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        // Header controls
                        ui.horizontal(|ui| {
                            if ui.button(if self.show_edit_mode { "View Mode" } else { "Edit Mode" }).clicked() {
                                self.show_edit_mode = !self.show_edit_mode;
                            }
                            ui.separator();
                            if ui.button("Close").clicked() {
                                should_close = true;
                            }
                        });

                        ui.separator();

                        // Main content area
                        if self.show_edit_mode {
                            // Edit mode
                            if let Some(task_mut) = &mut self.task {
                                show_task_editor(ui, task_mut, resources);
                                if ui.button("Save Changes").clicked() {
                                    action = Some(TaskAction::Update(task_mut.clone()));
                                }
                            }
                        } else {
                            // View mode
                            show_task_view(ui, &task);
                        }

                        ui.separator();
                        ui.heading("Comments");
                        
                        // Comments section
                        let mut comment_action = None;
                        show_comments_section(
                            ui,
                            &mut self.comments,
                            &mut self.new_comment_text,
                            &mut self.editing_comment_id,
                            &mut self.edit_comment_text,
                            task.id,
                            &mut comment_action
                        );
                        
                        if let Some(ca) = comment_action {
                            action = Some(ca);
                        }
                    });
                });
        }

        if should_close {
            self.close();
        }

        action
    }
}

fn show_task_view(ui: &mut Ui, task: &Task) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            // Title
            ui.heading(&task.title);
            
            // Status and Priority badges
            ui.horizontal(|ui| {
                draw_status_badge(ui, &task.status);
                ui.add_space(8.0);
                draw_priority_badge(ui, &task.priority);
            });

            ui.add_space(8.0);

            // Description
            if !task.description.is_empty() {
                ui.label("Description:");
                ui.group(|ui| {
                    ui.label(&task.description);
                });
            }

            // Metadata
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Created:");
                ui.label(task.created_at.format("%Y-%m-%d %H:%M").to_string());
            });

            if let Some(due) = task.due_date {
                ui.horizontal(|ui| {
                    ui.label("Due:");
                    ui.label(due.format("%Y-%m-%d %H:%M").to_string());
                    if task.is_overdue() {
                        ui.colored_label(Color32::RED, "(Overdue)");
                    }
                });
            }

            if let Some(hours) = task.estimated_hours {
                ui.horizontal(|ui| {
                    ui.label("Estimated:");
                    ui.label(format!("{:.1} hours", hours));
                });
            }

            // Tags
            if !task.tags.is_empty() {
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("Tags:");
                    for tag in &task.tags {
                        draw_tag(ui, tag);
                    }
                });
            }

            // Subtasks
            if !task.subtasks.is_empty() {
                ui.add_space(8.0);
                ui.label(format!("Subtasks ({}/{})", task.subtask_progress().0, task.subtask_progress().1));
                ui.group(|ui| {
                    for subtask in &task.subtasks {
                        ui.horizontal(|ui| {
                            ui.label(if subtask.completed { "☑" } else { "☐" });
                            if subtask.completed {
                                ui.colored_label(Color32::GRAY, &subtask.description);
                            } else {
                                ui.label(&subtask.description);
                            }
                        });
                    }
                });
            }
        });
    });
}

fn show_task_editor(ui: &mut Ui, task: &mut Task, resources: &[Resource]) {
    ui.group(|ui| {
        super::task_editor::show_task_editor(ui, task, resources);
    });
}

fn show_comments_section(
    ui: &mut Ui,
    comments: &mut Vec<Comment>,
    new_comment_text: &mut String,
    editing_comment_id: &mut Option<Uuid>,
    edit_comment_text: &mut String,
    task_id: Uuid,
    action: &mut Option<TaskAction>
) {
    // Display existing comments
    for comment in comments.iter() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&comment.author_name);
                ui.label(comment.created_at.format("%Y-%m-%d %H:%M").to_string());
                if comment.edited {
                    ui.label("(edited)");
                }
            });

            if *editing_comment_id == Some(comment.id) {
                // Show edit interface
                ui.text_edit_multiline(edit_comment_text);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        let mut updated_comment = comment.clone();
                        updated_comment.content = edit_comment_text.clone();
                        updated_comment.edited = true;
                        updated_comment.updated_at = Utc::now();
                        *action = Some(TaskAction::UpdateComment(updated_comment));
                        *editing_comment_id = None;
                    }
                    if ui.button("Cancel").clicked() {
                        *editing_comment_id = None;
                    }
                });
            } else {
                // Show comment content
                ui.label(&comment.content);
                ui.horizontal(|ui| {
                    if ui.small_button("Edit").clicked() {
                        *editing_comment_id = Some(comment.id);
                        *edit_comment_text = comment.content.clone();
                    }
                    if ui.small_button("Delete").clicked() {
                        *action = Some(TaskAction::DeleteComment(comment.id));
                    }
                });
            }
        });
        ui.add_space(4.0);
    }
    
    // Add new comment section
    ui.separator();
    ui.label("Add a comment:");
    ui.text_edit_multiline(new_comment_text);
    
    if ui.button("Post Comment").clicked() && !new_comment_text.is_empty() {
        let new_comment = Comment::new(
            task_id,
            EntityType::Task,
            "Current User".to_string(),
            new_comment_text.clone()
        );
        
        comments.push(new_comment.clone());
        new_comment_text.clear();
        *action = Some(TaskAction::AddComment(new_comment));
    }
}

fn draw_status_badge(ui: &mut Ui, status: &crate::domain::task::TaskStatus) {
    let (text, color) = match status {
        crate::domain::task::TaskStatus::Todo => ("TODO", Color32::GRAY),
        crate::domain::task::TaskStatus::InProgress => ("IN PROGRESS", Color32::from_rgb(100, 150, 255)),
        crate::domain::task::TaskStatus::Blocked => ("BLOCKED", Color32::from_rgb(255, 100, 100)),
        crate::domain::task::TaskStatus::Review => ("REVIEW", Color32::from_rgb(255, 200, 100)),
        crate::domain::task::TaskStatus::Done => ("DONE", Color32::from_rgb(100, 255, 100)),
        crate::domain::task::TaskStatus::Cancelled => ("CANCELLED", Color32::DARK_GRAY),
    };

    ui.colored_label(color, text);
}

fn draw_priority_badge(ui: &mut Ui, priority: &crate::domain::task::Priority) {
    let (text, color) = match priority {
        crate::domain::task::Priority::Low => ("LOW", Color32::from_rgb(100, 100, 100)),
        crate::domain::task::Priority::Medium => ("MEDIUM", Color32::from_rgb(100, 150, 255)),
        crate::domain::task::Priority::High => ("HIGH", Color32::from_rgb(255, 150, 100)),
        crate::domain::task::Priority::Critical => ("CRITICAL", Color32::from_rgb(255, 100, 100)),
    };

    ui.colored_label(color, text);
}

fn draw_tag(ui: &mut Ui, tag: &str) {
    ui.colored_label(Color32::from_rgb(150, 150, 200), format!("#{}", tag));
}

#[derive(Debug, Clone)]
pub enum TaskAction {
    Update(Task),
    AddComment(Comment),
    UpdateComment(Comment),
    DeleteComment(Uuid),
}