use crate::domain::task::{Task, TaskStatus, Priority};
use crate::services::TaskService;
use eframe::egui::{self, Ui, Context, Response, Rect, Pos2, Vec2, Color32, Stroke, Rounding, FontId, Align, Layout, Sense, CursorIcon};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;

pub struct KanbanView {
    pub(super) columns: Vec<KanbanColumn>,
    pub(super) drag_context: Option<DragContext>,
    pub(super) filter_options: FilterOptions,
    pub(super) view_preferences: ViewPreferences,
    pub(super) animations: AnimationManager,
    pub(super) quick_add_states: HashMap<String, QuickAddState>,
    pub(super) context_menu: Option<ContextMenu>,
    pub(super) selected_cards: HashSet<Uuid>,
    pub(super) hovered_card: Option<Uuid>,
    pub(super) focused_card: Option<Uuid>,
    pub(super) swimlane_config: SwimlaneConfig,
    pub(super) tag_colors: HashMap<String, Color32>,
}

#[derive(Clone)]
pub struct KanbanColumn {
    pub title: String,
    pub status: TaskStatus,
    pub color: Color32,
    pub width: f32,
    pub collapsed: bool,
    pub wip_limit: Option<usize>,
    pub visible: bool,
    pub position: usize,
}

pub struct DragContext {
    pub task_id: Uuid,
    pub start_position: Pos2,
    pub current_position: Pos2,
    pub selected_tasks: Vec<Uuid>,
    pub is_multi_drag: bool,
    pub original_status: TaskStatus,
}

#[derive(Default, Clone)]
pub struct FilterOptions {
    pub search_text: Option<String>,
    pub tags: Vec<String>,
    pub priorities: Vec<Priority>,
    pub assigned_to: Option<Uuid>,
    pub due_date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub show_blocked: bool,
    pub show_completed: bool,
}

pub(super) struct ViewPreferences {
    pub(super) column_widths: HashMap<String, f32>,
    pub(super) wip_limits: HashMap<String, usize>,
    pub(super) collapsed_columns: HashSet<String>,
    pub(super) hidden_columns: HashSet<String>,
    pub(super) swimlanes_enabled: bool,
    pub(super) swimlane_type: SwimlaneType,
    pub(super) theme: ThemeConfig,
}

pub(super) struct AnimationManager {
    pub(super) card_animations: HashMap<Uuid, CardAnimation>,
    pub(super) column_animations: HashMap<String, ColumnAnimation>,
    pub(super) time: f32,
}

pub(super) struct CardAnimation {
    pub(super) start_pos: Pos2,
    pub(super) end_pos: Pos2,
    pub(super) start_time: f32,
    pub(super) duration: f32,
    pub(super) opacity: f32,
    pub(super) scale: f32,
}

pub(super) struct ColumnAnimation {
    pub(super) start_width: f32,
    pub(super) end_width: f32,
    pub(super) start_time: f32,
    pub(super) duration: f32,
}

pub struct QuickAddState {
    pub(super) visible: bool,
    pub(super) title: String,
    pub(super) metadata: QuickAddMetadata,
}

#[derive(Default, Clone)]
pub struct QuickAddMetadata {
    pub title: String,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub description: Option<String>,
}

pub(super) struct ContextMenu {
    pub(super) task_id: Uuid,
    pub(super) position: Pos2,
    pub(super) items: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SwimlaneType {
    None,
    Priority,
    Assignee,
    Tag,
}

pub(super) struct SwimlaneConfig {
    pub(super) enabled: bool,
    pub(super) swimlane_type: SwimlaneType,
    pub(super) collapsed_lanes: HashSet<String>,
    pub(super) lane_order: Vec<String>,
}

pub(super) struct ThemeConfig {
    pub(super) card_shadow: bool,
    pub(super) animations_enabled: bool,
    pub(super) compact_mode: bool,
    pub(super) color_scheme: ColorScheme,
}

#[derive(Clone, Copy)]
enum ColorScheme {
    Light,
    Dark,
    HighContrast,
}

pub struct WipLimit {
    limit: usize,
    strict: bool,
}

pub struct CardStyle {
    pub border_color: Color32,
    pub border_width: f32,
    pub background_color: Color32,
    pub shadow_blur: f32,
    pub shadow_offset: Vec2,
    pub elevation: f32,
    pub priority_indicator_color: Color32,
    pub show_overdue_badge: bool,
    pub overdue_badge_color: Color32,
    pub pulse_animation: bool,
    pub show_blocked_overlay: bool,
    pub blocked_pattern: String,
    pub opacity: f32,
}

impl KanbanView {
    pub fn new() -> Self {
        let columns = vec![
            KanbanColumn {
                title: "To Do".to_string(),
                status: TaskStatus::Todo,
                color: Color32::from_rgb(200, 200, 200),
                width: 250.0,
                collapsed: false,
                wip_limit: None,
                visible: true,
                position: 0,
            },
            KanbanColumn {
                title: "In Progress".to_string(),
                status: TaskStatus::InProgress,
                color: Color32::from_rgb(100, 150, 255),
                width: 250.0,
                collapsed: false,
                wip_limit: Some(3),
                visible: true,
                position: 1,
            },
            KanbanColumn {
                title: "Review".to_string(),
                status: TaskStatus::Review,
                color: Color32::from_rgb(255, 200, 100),
                width: 250.0,
                collapsed: false,
                wip_limit: Some(2),
                visible: true,
                position: 2,
            },
            KanbanColumn {
                title: "Done".to_string(),
                status: TaskStatus::Done,
                color: Color32::from_rgb(100, 255, 100),
                width: 250.0,
                collapsed: false,
                wip_limit: None,
                visible: true,
                position: 3,
            },
        ];

        Self {
            columns,
            drag_context: None,
            filter_options: FilterOptions::default(),
            view_preferences: ViewPreferences {
                column_widths: HashMap::new(),
                wip_limits: HashMap::new(),
                collapsed_columns: HashSet::new(),
                hidden_columns: HashSet::new(),
                swimlanes_enabled: false,
                swimlane_type: SwimlaneType::None,
                theme: ThemeConfig {
                    card_shadow: true,
                    animations_enabled: true,
                    compact_mode: false,
                    color_scheme: ColorScheme::Light,
                },
            },
            animations: AnimationManager {
                card_animations: HashMap::new(),
                column_animations: HashMap::new(),
                time: 0.0,
            },
            quick_add_states: HashMap::new(),
            context_menu: None,
            selected_cards: HashSet::new(),
            hovered_card: None,
            focused_card: None,
            swimlane_config: SwimlaneConfig {
                enabled: false,
                swimlane_type: SwimlaneType::None,
                collapsed_lanes: HashSet::new(),
                lane_order: Vec::new(),
            },
            tag_colors: HashMap::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>) {
        let filtered_tasks = self.apply_filters(tasks, &self.filter_options.clone());
        
        ui.vertical(|ui| {
            self.show_header(ui);
            self.show_filters(ui);
            
            if self.swimlane_config.enabled {
                self.show_with_swimlanes(ui, &filtered_tasks);
            } else {
                self.show_board(ui, &filtered_tasks);
            }
        });

        self.handle_drag_drop(ui.ctx());
        self.update_animations(ui.ctx().input(|i| i.unstable_dt));
    }

    fn show_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("📋 Kanban Board");
            
            ui.separator();
            
            if ui.button("➕ Add Task").clicked() {
                self.show_quick_add("Todo");
            }
            
            ui.menu_button("👁 View", |ui| {
                ui.checkbox(&mut self.swimlane_config.enabled, "Enable Swimlanes");
                ui.checkbox(&mut self.view_preferences.theme.compact_mode, "Compact Mode");
                ui.checkbox(&mut self.view_preferences.theme.animations_enabled, "Animations");
                
                ui.separator();
                
                ui.label("Swimlane Type:");
                ui.radio_value(&mut self.swimlane_config.swimlane_type, SwimlaneType::None, "None");
                ui.radio_value(&mut self.swimlane_config.swimlane_type, SwimlaneType::Priority, "Priority");
                ui.radio_value(&mut self.swimlane_config.swimlane_type, SwimlaneType::Assignee, "Assignee");
                ui.radio_value(&mut self.swimlane_config.swimlane_type, SwimlaneType::Tag, "Tag");
            });
            
            ui.menu_button("⚙ Settings", |ui| {
                ui.label("WIP Limits:");
                for column in &mut self.columns {
                    ui.horizontal(|ui| {
                        ui.label(&column.title);
                        if let Some(limit) = &mut column.wip_limit {
                            ui.add(egui::DragValue::new(limit).speed(1));
                        } else {
                            if ui.button("Set Limit").clicked() {
                                column.wip_limit = Some(5);
                            }
                        }
                    });
                }
            });
        });
    }

    fn show_filters(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_response = ui.text_edit_singleline(
                self.filter_options.search_text.get_or_insert(String::new())
            );
            
            if search_response.changed() {
                // Trigger search
            }
            
            ui.separator();
            
            ui.menu_button("🏷 Tags", |ui| {
                let all_tags = self.get_all_tags();
                for tag in all_tags {
                    let mut selected = self.filter_options.tags.contains(&tag);
                    if ui.checkbox(&mut selected, &tag).changed() {
                        if selected {
                            self.filter_options.tags.push(tag.clone());
                        } else {
                            self.filter_options.tags.retain(|t| t != &tag);
                        }
                    }
                }
            });
            
            ui.menu_button("🎯 Priority", |ui| {
                for priority in [Priority::Critical, Priority::High, Priority::Medium, Priority::Low] {
                    let mut selected = self.filter_options.priorities.contains(&priority);
                    if ui.checkbox(&mut selected, format!("{:?}", priority)).changed() {
                        if selected {
                            self.filter_options.priorities.push(priority);
                        } else {
                            self.filter_options.priorities.retain(|p| *p != priority);
                        }
                    }
                }
            });
            
            if ui.button("Clear Filters").clicked() {
                self.filter_options = FilterOptions::default();
            }
        });
    }

    fn show_board(&mut self, ui: &mut Ui, tasks: &[Task]) {
        egui::ScrollArea::horizontal()
            .id_source("kanban_board_horizontal")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0; // Add spacing between columns
                    
                    for column in self.columns.clone().iter() {
                        if !column.visible {
                            continue;
                        }
                        
                        self.show_column(ui, column, tasks);
                    }
                });
            });
    }

    fn show_column(&mut self, ui: &mut Ui, column: &KanbanColumn, tasks: &[Task]) {
        let column_tasks: Vec<&Task> = tasks.iter()
            .filter(|t| t.status == column.status)
            .collect();
        
        let is_over_wip = if let Some(limit) = column.wip_limit {
            column_tasks.len() > limit
        } else {
            false
        };
        
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(if column.collapsed { 50.0 } else { column.width });
                ui.set_max_width(if column.collapsed { 50.0 } else { column.width });
                
                // Column header with background
                ui.group(|ui| {
                    self.show_column_header(ui, column, column_tasks.len(), is_over_wip);
                });
                
                if !column.collapsed {
                    ui.add_space(4.0);
                    
                    egui::ScrollArea::vertical()
                        .id_source(format!("column_scroll_{}", column.title))
                        .max_height(600.0)
                        .show(ui, |ui| {
                            ui.set_min_width(column.width - 20.0);
                            
                            let drop_target = ui.allocate_response(
                                Vec2::new(column.width - 10.0, 10.0),
                                Sense::hover()
                            );
                            
                            if self.is_dragging() && drop_target.hovered() {
                                ui.painter().rect_filled(
                                    drop_target.rect,
                                    Rounding::same(4.0),
                                    Color32::from_rgba_premultiplied(100, 150, 255, 50)
                                );
                            }
                            
                            for task in column_tasks {
                                self.show_enhanced_task_card(ui, task, column);
                                ui.add_space(4.0);
                            }
                            
                            if self.quick_add_states.get(&column.title).map_or(false, |s| s.visible) {
                                self.show_quick_add_form(ui, &column.title);
                            }
                        });
                }
            });
        });
    }

    fn show_column_header(&mut self, ui: &mut Ui, column: &KanbanColumn, task_count: usize, is_over_wip: bool) {
        let header_color = if is_over_wip {
            Color32::from_rgb(255, 200, 0)
        } else {
            column.color
        };
        
        ui.horizontal(|ui| {
            ui.set_max_width(if column.collapsed { 50.0 } else { column.width });
            
            if ui.button(if column.collapsed { "▶" } else { "▼" }).clicked() {
                self.toggle_column_collapse(&column.title);
            }
            
            if !column.collapsed {
                ui.colored_label(header_color, &column.title);
                ui.label(format!("({})", task_count));
                
                if let Some(limit) = column.wip_limit {
                    let wip_text = format!("{}/{}", task_count, limit);
                    let wip_color = if task_count > limit {
                        Color32::RED
                    } else if task_count == limit {
                        Color32::YELLOW
                    } else {
                        Color32::GREEN
                    };
                    ui.colored_label(wip_color, wip_text);
                }
                
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("➕").on_hover_text("Quick Add").clicked() {
                        self.show_quick_add(&column.title);
                    }
                });
            }
        });
    }

    fn show_enhanced_task_card(&mut self, ui: &mut Ui, task: &Task, column: &KanbanColumn) {
        let card_id = ui.make_persistent_id(task.id);
        let is_selected = self.selected_cards.contains(&task.id);
        let is_hovered = self.hovered_card == Some(task.id);
        let style = self.get_card_style(task);
        
        let card_response = ui.group(|ui| {
            ui.set_min_width(column.width - 20.0);
            
            if style.show_blocked_overlay && task.status == TaskStatus::Blocked {
                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    Rounding::same(4.0),
                    Color32::from_rgba_premultiplied(255, 0, 0, 30)
                );
            }
            
            ui.horizontal(|ui| {
                let priority_color = self.get_priority_color(task.priority);
                ui.painter().rect_filled(
                    Rect::from_min_size(ui.cursor().min, Vec2::new(4.0, 60.0)),
                    Rounding::ZERO,
                    priority_color
                );
                ui.add_space(8.0);
                
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if is_selected {
                            ui.label("✓");
                        }
                        
                        let title_response = ui.label(
                            egui::RichText::new(&task.title)
                                .strong()
                                .size(14.0)
                        );
                        
                        if task.is_overdue() {
                            ui.colored_label(Color32::RED, "⚠ OVERDUE");
                        }
                    });
                    
                    if !task.description.is_empty() && !self.view_preferences.theme.compact_mode {
                        let desc_preview = if task.description.len() > 100 {
                            format!("{}...", &task.description[..100])
                        } else {
                            task.description.clone()
                        };
                        ui.label(
                            egui::RichText::new(desc_preview)
                                .small()
                                .color(Color32::GRAY)
                        );
                    }
                    
                    if !task.subtasks.is_empty() {
                        let (completed, total) = task.subtask_progress();
                        let progress = completed as f32 / total as f32;
                        
                        ui.horizontal(|ui| {
                            ui.label(format!("📊 {}/{}", completed, total));
                            
                            let progress_rect = ui.available_rect_before_wrap();
                            let progress_width = 100.0;
                            let progress_height = 6.0;
                            
                            ui.painter().rect_filled(
                                Rect::from_min_size(
                                    progress_rect.min,
                                    Vec2::new(progress_width, progress_height)
                                ),
                                Rounding::same(3.0),
                                Color32::from_gray(200)
                            );
                            
                            ui.painter().rect_filled(
                                Rect::from_min_size(
                                    progress_rect.min,
                                    Vec2::new(progress_width * progress, progress_height)
                                ),
                                Rounding::same(3.0),
                                Color32::from_rgb(52, 199, 89)
                            );
                            
                            ui.add_space(progress_width + 5.0);
                        });
                    }
                    
                    if !task.tags.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            for tag in &task.tags {
                                let tag_color = self.get_or_assign_tag_color(tag);
                                ui.label(
                                    egui::RichText::new(format!("#{}", tag))
                                        .small()
                                        .color(tag_color)
                                        .background_color(Color32::from_rgba_premultiplied(
                                            tag_color.r(),
                                            tag_color.g(),
                                            tag_color.b(),
                                            30
                                        ))
                                );
                            }
                        });
                    }
                    
                    ui.horizontal(|ui| {
                        if let Some(due) = task.due_date {
                            let days_until = (due.date_naive() - Utc::now().date_naive()).num_days();
                            let due_color = if days_until < 0 {
                                Color32::RED
                            } else if days_until <= 3 {
                                Color32::YELLOW
                            } else {
                                Color32::GRAY
                            };
                            ui.colored_label(due_color, format!("📅 {}", due.format("%b %d")));
                        }
                        
                        if task.assigned_resource_id.is_some() {
                            ui.label("👤");
                        }
                        
                        if task.estimated_hours.is_some() {
                            ui.label(format!("⏱ {}h", task.estimated_hours.unwrap()));
                        }
                    });
                });
            });
        });
        
        let response = card_response.response;
        
        if response.hovered() {
            self.hovered_card = Some(task.id);
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        
        if response.clicked() {
            let multi_select = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
            self.select_card(task.id, multi_select);
        }
        
        if response.double_clicked() {
            self.handle_card_double_click(task.id);
        }
        
        if response.secondary_clicked() {
            self.show_context_menu(task.id, response.interact_pointer_pos().unwrap_or_default());
        }
        
        if response.drag_started() {
            self.start_drag(task.id, response.interact_pointer_pos().unwrap_or_default());
        }
        
        if self.is_dragging() && response.hovered() {
            response.on_hover_ui(|ui| {
                ui.label("Drop here");
            });
        }
    }

    fn show_with_swimlanes(&mut self, ui: &mut Ui, tasks: &[Task]) {
        let swimlanes = self.organize_into_swimlanes(tasks);
        
        egui::ScrollArea::both()
            .id_source("kanban_swimlanes")
            .show(ui, |ui| {
                for (lane_name, lane_tasks) in swimlanes {
                    ui.collapsing(&lane_name, |ui| {
                        self.show_board(ui, &lane_tasks);
                    });
                    ui.separator();
                }
            });
    }

    fn show_quick_add_form(&mut self, ui: &mut Ui, column_title: &str) {
        if let Some(state) = self.quick_add_states.get_mut(column_title) {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    let response = ui.text_edit_singleline(&mut state.title);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Create task
                        state.visible = false;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        state.visible = false;
                    }
                });
            });
        }
    }

    pub fn get_card_style(&self, task: &Task) -> CardStyle {
        let is_hovered = self.hovered_card == Some(task.id);
        let is_critical = task.priority == Priority::Critical;
        let is_blocked = task.status == TaskStatus::Blocked;
        
        CardStyle {
            border_color: if is_critical {
                Color32::from_rgb(255, 0, 0)
            } else {
                Color32::from_gray(200)
            },
            border_width: if is_critical { 3.0 } else { 1.0 },
            background_color: Color32::WHITE,
            shadow_blur: if is_hovered { 12.0 } else { 4.0 },
            shadow_offset: if is_hovered { Vec2::new(0.0, 4.0) } else { Vec2::new(0.0, 2.0) },
            elevation: if is_hovered { 2.0 } else { 0.0 },
            priority_indicator_color: self.get_priority_color(task.priority),
            show_overdue_badge: task.is_overdue(),
            overdue_badge_color: Color32::from_rgb(255, 59, 48),
            pulse_animation: task.is_overdue(),
            show_blocked_overlay: is_blocked,
            blocked_pattern: "diagonal_stripes".to_string(),
            opacity: if is_blocked { 0.8 } else { 1.0 },
        }
    }

    fn get_priority_color(&self, priority: Priority) -> Color32 {
        match priority {
            Priority::Critical => Color32::from_rgb(255, 59, 48),
            Priority::High => Color32::from_rgb(255, 149, 0),
            Priority::Medium => Color32::from_rgb(52, 199, 89),
            Priority::Low => Color32::from_rgb(175, 175, 175),
        }
    }

    pub(super) fn get_or_assign_tag_color(&mut self, tag: &str) -> Color32 {
        if !self.tag_colors.contains_key(tag) {
            let hash = tag.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
            let hue = (hash % 360) as f32;
            let color = self.hsv_to_rgb(hue, 0.7, 0.8);
            self.tag_colors.insert(tag.to_string(), color);
        }
        *self.tag_colors.get(tag).unwrap()
    }

    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> Color32 {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        Color32::from_rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8
        )
    }

    fn toggle_column_collapse(&mut self, column_title: &str) {
        if let Some(column) = self.columns.iter_mut().find(|c| c.title == column_title) {
            column.collapsed = !column.collapsed;
            
            let start_width = if column.collapsed { column.width } else { 50.0 };
            let end_width = if column.collapsed { 50.0 } else { 250.0 };
            
            self.start_column_expand_animation(column_title, start_width, end_width);
        }
    }

    pub fn apply_filters(&self, tasks: &[Task], filter: &FilterOptions) -> Vec<Task> {
        let mut filtered = tasks.to_vec();
        
        if let Some(search) = &filter.search_text {
            if !search.is_empty() {
                let search_lower = search.to_lowercase();
                filtered.retain(|t| 
                    t.title.to_lowercase().contains(&search_lower) ||
                    t.description.to_lowercase().contains(&search_lower)
                );
            }
        }
        
        if !filter.tags.is_empty() {
            filtered.retain(|t| 
                filter.tags.iter().any(|tag| t.tags.contains(tag))
            );
        }
        
        if !filter.priorities.is_empty() {
            filtered.retain(|t| filter.priorities.contains(&t.priority));
        }
        
        if let Some(assignee) = filter.assigned_to {
            filtered.retain(|t| t.assigned_resource_id == Some(assignee));
        }
        
        if let Some((start, end)) = filter.due_date_range {
            filtered.retain(|t| {
                if let Some(due) = t.due_date {
                    due >= start && due <= end
                } else {
                    false
                }
            });
        }
        
        if !filter.show_blocked {
            filtered.retain(|t| t.status != TaskStatus::Blocked);
        }
        
        if !filter.show_completed {
            filtered.retain(|t| t.status != TaskStatus::Done);
        }
        
        filtered
    }

    pub fn organize_into_swimlanes(&self, tasks: &[Task]) -> HashMap<String, Vec<Task>> {
        let mut swimlanes = HashMap::new();
        
        match self.swimlane_config.swimlane_type {
            SwimlaneType::Priority => {
                for task in tasks {
                    let lane = format!("{:?}", task.priority);
                    swimlanes.entry(lane).or_insert(Vec::new()).push(task.clone());
                }
            },
            SwimlaneType::Assignee => {
                for task in tasks {
                    let lane = task.assigned_resource_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "Unassigned".to_string());
                    swimlanes.entry(lane).or_insert(Vec::new()).push(task.clone());
                }
            },
            SwimlaneType::Tag => {
                for task in tasks {
                    if task.tags.is_empty() {
                        swimlanes.entry("No Tags".to_string()).or_insert(Vec::new()).push(task.clone());
                    } else {
                        for tag in &task.tags {
                            swimlanes.entry(tag.clone()).or_insert(Vec::new()).push(task.clone());
                        }
                    }
                }
            },
            _ => {
                swimlanes.insert("All Tasks".to_string(), tasks.to_vec());
            }
        }
        
        swimlanes
    }

    fn get_all_tags(&self) -> Vec<String> {
        vec!["frontend".to_string(), "backend".to_string(), "bug".to_string(), "feature".to_string()]
    }

    pub fn start_drag(&mut self, task_id: Uuid, start_position: Pos2) {
        let selected = if self.selected_cards.contains(&task_id) {
            self.selected_cards.iter().cloned().collect()
        } else {
            vec![task_id]
        };
        
        self.drag_context = Some(DragContext {
            task_id,
            start_position,
            current_position: start_position,
            selected_tasks: selected.clone(),
            is_multi_drag: selected.len() > 1,
            original_status: TaskStatus::Todo,
        });
    }

    pub fn update_drag_position(&mut self, position: Pos2) {
        if let Some(ctx) = &mut self.drag_context {
            ctx.current_position = position;
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_context.is_some()
    }

    pub fn cancel_drag(&mut self) {
        self.drag_context = None;
    }

    pub fn get_drag_context(&self) -> Option<&DragContext> {
        self.drag_context.as_ref()
    }

    pub fn handle_drag_drop(&mut self, ctx: &Context) {
        if let Some(drag_ctx) = &self.drag_context {
            ctx.input(|i| {
                if i.pointer.any_released() {
                    self.drag_context = None;
                }
                if i.key_pressed(egui::Key::Escape) {
                    self.cancel_drag();
                }
            });
        }
    }

    pub fn update_animations(&mut self, dt: f32) {
        self.animations.time += dt;
        
        self.animations.card_animations.retain(|_, anim| {
            let progress = (self.animations.time - anim.start_time) / anim.duration;
            progress < 1.0
        });
        
        self.animations.column_animations.retain(|_, anim| {
            let progress = (self.animations.time - anim.start_time) / anim.duration;
            progress < 1.0
        });
    }

    pub fn start_column_expand_animation(&mut self, column_title: &str, start_width: f32, end_width: f32) {
        self.animations.column_animations.insert(
            column_title.to_string(),
            ColumnAnimation {
                start_width,
                end_width,
                start_time: self.animations.time,
                duration: 0.3,
            }
        );
    }

    pub fn select_card(&mut self, task_id: Uuid, multi_select: bool) {
        if multi_select {
            if self.selected_cards.contains(&task_id) {
                self.selected_cards.remove(&task_id);
            } else {
                self.selected_cards.insert(task_id);
            }
        } else {
            self.selected_cards.clear();
            self.selected_cards.insert(task_id);
        }
    }

    pub fn select_multiple_tasks(&mut self, task_ids: Vec<Uuid>) {
        self.selected_cards = task_ids.into_iter().collect();
    }

    pub fn handle_card_double_click(&mut self, task_id: Uuid) {
        // Open edit dialog
        self.focused_card = Some(task_id);
    }

    pub fn show_context_menu(&mut self, task_id: Uuid, position: Pos2) {
        self.context_menu = Some(ContextMenu {
            task_id,
            position,
            items: vec![
                "Edit".to_string(),
                "Delete".to_string(),
                "Duplicate".to_string(),
                "Move to".to_string(),
            ],
        });
    }

    pub fn show_quick_add(&mut self, column_title: &str) {
        self.quick_add_states.insert(
            column_title.to_string(),
            QuickAddState {
                visible: true,
                title: String::new(),
                metadata: QuickAddMetadata::default(),
            }
        );
    }

    pub fn set_hovered_card(&mut self, card_id: Option<Uuid>) {
        self.hovered_card = card_id;
    }

    pub fn get_columns(&self) -> &Vec<KanbanColumn> {
        &self.columns
    }

    pub fn set_wip_limit(&mut self, column_title: &str, limit: usize) {
        if let Some(column) = self.columns.iter_mut().find(|c| c.title == column_title) {
            column.wip_limit = Some(limit);
        }
    }

    pub fn enable_swimlanes_by_priority(&mut self) {
        self.swimlane_config.enabled = true;
        self.swimlane_config.swimlane_type = SwimlaneType::Priority;
    }

    pub fn enable_swimlanes_by_assignee(&mut self) {
        self.swimlane_config.enabled = true;
        self.swimlane_config.swimlane_type = SwimlaneType::Assignee;
    }
}

impl Default for ViewPreferences {
    fn default() -> Self {
        Self {
            column_widths: HashMap::new(),
            wip_limits: HashMap::new(),
            collapsed_columns: HashSet::new(),
            hidden_columns: HashSet::new(),
            swimlanes_enabled: false,
            swimlane_type: SwimlaneType::None,
            theme: ThemeConfig {
                card_shadow: true,
                animations_enabled: true,
                compact_mode: false,
                color_scheme: ColorScheme::Light,
            },
        }
    }
}