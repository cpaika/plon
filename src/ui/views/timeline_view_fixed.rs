use crate::domain::{task::Task, goal::Goal, resource::Resource, dependency::{Dependency, DependencyGraph, DependencyType}};
use crate::services::timeline_scheduler::{TimelineScheduler, TimelineSchedule, TaskSchedule};
use crate::ui::widgets::gantt_chart::GanttChart;
use eframe::egui::{self, Ui, ScrollArea, Rect, Sense, Vec2, Pos2, Color32, Stroke};
use chrono::{Utc, Duration, NaiveDate, Local};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineViewMode {
    Gantt,
    List,
    Calendar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineFilter {
    All,
    InProgress,
    Completed,
    Unassigned,
    Overdue,
}

pub struct TimelineProcessedData {
    pub task_count: usize,
    pub resource_count: usize,
    pub unassigned_tasks: usize,
}

pub struct TimelineView {
    pub days_to_show: i64,
    pub show_gantt: bool,
    pub show_resources: bool,
    pub selected_view: TimelineViewMode,
    pub start_date: NaiveDate,  // Made public for testing
    filter: TimelineFilter,
    pub gantt_chart: GanttChart,  // Made public for testing
    scheduler: TimelineScheduler,
    
    // State management to prevent jumping
    scroll_offset_x: f32,
    scroll_offset_y: f32,
    cached_schedule: Option<TimelineSchedule>,
    last_task_count: usize,
    zoom_level: f32,
    selected_task_id: Option<Uuid>,
}

impl Default for TimelineView {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineView {
    pub fn new() -> Self {
        Self {
            days_to_show: 30,
            show_gantt: true,
            show_resources: true,
            selected_view: TimelineViewMode::Gantt,
            filter: TimelineFilter::All,
            gantt_chart: GanttChart::new(),
            scheduler: TimelineScheduler::new(),
            start_date: Local::now().naive_local().date(),
            
            // Initialize state management fields
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
            cached_schedule: None,
            last_task_count: 0,
            zoom_level: 1.0,
            selected_task_id: None,
        }
    }
    
    pub fn set_view_mode(&mut self, mode: TimelineViewMode) {
        // Only change the mode, preserve all other state
        self.selected_view = mode;
    }
    
    pub fn set_filter(&mut self, filter: TimelineFilter) {
        // Only change the filter, preserve scroll position
        self.filter = filter;
    }
    
    pub fn set_date_range(&mut self, days: i64) {
        let new_days = days.max(7).min(365);
        
        // Adjust scroll position proportionally when zooming
        let zoom_factor = new_days as f32 / self.days_to_show as f32;
        self.scroll_offset_x *= zoom_factor;
        
        self.days_to_show = new_days;
        self.gantt_chart.set_days_to_show(self.days_to_show);
    }
    
    pub fn scroll_to_today(&mut self) {
        let today = Local::now().naive_local().date();
        let days_from_start = (today - self.start_date).num_days();
        self.scroll_offset_x = (days_from_start as f32 * 30.0).max(0.0);
    }
    
    pub fn reset_view(&mut self) {
        self.scroll_offset_x = 0.0;
        self.scroll_offset_y = 0.0;
        self.zoom_level = 1.0;
        self.days_to_show = 30;
        self.start_date = Local::now().naive_local().date();
        self.gantt_chart.set_days_to_show(30);
    }
    
    pub fn process_timeline_data(&self, tasks: &HashMap<Uuid, Task>, resources: &HashMap<Uuid, Resource>) -> TimelineProcessedData {
        let unassigned_tasks = tasks.values()
            .filter(|t| t.assigned_resource_id.is_none())
            .count();
        
        TimelineProcessedData {
            task_count: tasks.len(),
            resource_count: resources.len(),
            unassigned_tasks,
        }
    }
    
    pub fn apply_filters(&self, tasks: &HashMap<Uuid, Task>) -> HashMap<Uuid, Task> {
        tasks.iter()
            .filter(|(_, task)| {
                match self.filter {
                    TimelineFilter::All => true,
                    TimelineFilter::InProgress => task.status == crate::domain::task::TaskStatus::InProgress,
                    TimelineFilter::Completed => task.status == crate::domain::task::TaskStatus::Done,
                    TimelineFilter::Unassigned => task.assigned_resource_id.is_none(),
                    TimelineFilter::Overdue => task.is_overdue(),
                }
            })
            .map(|(id, task)| (*id, task.clone()))
            .collect()
    }
    
    pub fn assign_resource_to_task(&self, task: &mut Task, resource_id: Uuid) {
        task.assigned_resource_id = Some(resource_id);
        task.updated_at = Utc::now();
    }
    
    pub fn create_dependency(
        &self,
        from_task_id: Uuid,
        to_task_id: Uuid,
        dep_type: DependencyType,
        graph: &mut DependencyGraph,
    ) -> bool {
        let dependency = Dependency::new(from_task_id, to_task_id, dep_type);
        graph.add_dependency(&dependency).is_ok()
    }
    
    pub fn calculate_schedule(
        &mut self,
        tasks: &HashMap<Uuid, Task>,
        resources: &HashMap<Uuid, Resource>,
        graph: &DependencyGraph,
    ) -> Result<TimelineSchedule, String> {
        // Use cached schedule if task count hasn't changed
        if tasks.len() == self.last_task_count {
            if let Some(ref cached) = self.cached_schedule {
                return Ok(cached.clone());
            }
        }
        
        let schedule = self.scheduler.calculate_schedule(tasks, resources, graph, self.start_date)?;
        self.cached_schedule = Some(schedule.clone());
        self.last_task_count = tasks.len();
        Ok(schedule)
    }
    
    pub fn is_task_critical(&self, task_id: Uuid, critical_path: &[Uuid]) -> bool {
        critical_path.contains(&task_id)
    }
    
    pub fn group_tasks_by_goal(
        &self,
        tasks: &HashMap<Uuid, Task>,
        goals: &HashMap<Uuid, Goal>,
    ) -> HashMap<Option<Uuid>, Vec<Uuid>> {
        let mut grouped = HashMap::new();
        
        for (task_id, task) in tasks {
            grouped.entry(task.goal_id)
                .or_insert_with(Vec::new)
                .push(*task_id);
        }
        
        grouped
    }
    
    pub fn generate_warnings(&self, tasks: &HashMap<Uuid, Task>) -> Vec<String> {
        let mut warnings = Vec::new();
        
        for task in tasks.values() {
            // Check for overdue tasks
            if task.is_overdue() {
                warnings.push(format!("Task '{}' is overdue", task.title));
            }
            
            // Check for unassigned tasks with estimates
            if task.assigned_resource_id.is_none() && task.estimated_hours.is_some() {
                warnings.push(format!("Task '{}' has an estimate but is unassigned", task.title));
            }
            
            // Check for assigned tasks without estimates
            if task.assigned_resource_id.is_some() && task.estimated_hours.is_none() {
                warnings.push(format!("Task '{}' is assigned but has no estimate", task.title));
            }
        }
        
        warnings
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &[Task], goals: &[Goal]) {
        ui.heading("📅 Timeline View");
        
        // Top toolbar
        ui.horizontal(|ui| {
            // View mode selector
            ui.label("View:");
            if ui.selectable_label(self.selected_view == TimelineViewMode::Gantt, "📊 Gantt").clicked() {
                self.set_view_mode(TimelineViewMode::Gantt);
            }
            if ui.selectable_label(self.selected_view == TimelineViewMode::List, "📋 List").clicked() {
                self.set_view_mode(TimelineViewMode::List);
            }
            if ui.selectable_label(self.selected_view == TimelineViewMode::Calendar, "📅 Calendar").clicked() {
                self.set_view_mode(TimelineViewMode::Calendar);
            }
            
            ui.separator();
            
            // Zoom controls
            ui.label("Zoom:");
            if ui.button("🔍-").clicked() {
                self.set_date_range((self.days_to_show as f32 * 1.5) as i64);
            }
            if ui.button("🔍+").clicked() {
                self.set_date_range((self.days_to_show as f32 / 1.5) as i64);
            }
            
            // Date range slider
            ui.label("Days:");
            let mut days = self.days_to_show;
            if ui.add(egui::Slider::new(&mut days, 7..=365).logarithmic(true)).changed() {
                self.set_date_range(days);
            }
            
            ui.separator();
            
            // Navigation buttons
            if ui.button("⬅").clicked() {
                self.scroll_offset_x = (self.scroll_offset_x - 100.0).max(0.0);
            }
            if ui.button("Today").clicked() {
                self.scroll_to_today();
            }
            if ui.button("➡").clicked() {
                self.scroll_offset_x += 100.0;
            }
            
            if ui.button("Reset").clicked() {
                self.reset_view();
            }
        });
        
        // Filter bar
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui.selectable_label(self.filter == TimelineFilter::All, "All").clicked() {
                self.set_filter(TimelineFilter::All);
            }
            ui.separator();
            if ui.selectable_label(self.filter == TimelineFilter::InProgress, "🔄 In Progress").clicked() {
                self.set_filter(TimelineFilter::InProgress);
            }
            if ui.selectable_label(self.filter == TimelineFilter::Completed, "✅ Completed").clicked() {
                self.set_filter(TimelineFilter::Completed);
            }
            if ui.selectable_label(self.filter == TimelineFilter::Unassigned, "❓ Unassigned").clicked() {
                self.set_filter(TimelineFilter::Unassigned);
            }
            if ui.selectable_label(self.filter == TimelineFilter::Overdue, "⚠️ Overdue").clicked() {
                self.set_filter(TimelineFilter::Overdue);
            }
        });
        
        ui.separator();
        
        // Main content area with preserved scroll position
        ScrollArea::both()
            .auto_shrink([false, false])
            .scroll_offset(Vec2::new(self.scroll_offset_x, self.scroll_offset_y))
            .show(ui, |ui| {
                // Store scroll position for next frame
                let scroll_delta = ui.input(|i| i.scroll_delta);
                self.scroll_offset_x = (self.scroll_offset_x - scroll_delta.x).max(0.0);
                self.scroll_offset_y = (self.scroll_offset_y - scroll_delta.y).max(0.0);
                
                match self.selected_view {
                    TimelineViewMode::Gantt => {
                        self.show_gantt_view(ui, tasks, goals);
                    }
                    TimelineViewMode::List => {
                        self.show_list_view(ui, tasks);
                    }
                    TimelineViewMode::Calendar => {
                        self.show_calendar_view(ui, tasks);
                    }
                }
            });
    }
    
    fn show_gantt_view(&mut self, ui: &mut Ui, tasks: &[Task], goals: &[Goal]) {
        // Convert to HashMaps for processing
        let task_map: HashMap<Uuid, Task> = tasks.iter()
            .map(|t| (t.id, t.clone()))
            .collect();
        
        let filtered_tasks = self.apply_filters(&task_map);
        
        if filtered_tasks.is_empty() {
            ui.label("No tasks match the current filter");
            return;
        }
        
        // Simple Gantt visualization
        let row_height = 30.0;
        let day_width = 25.0 * self.zoom_level;
        let label_width = 200.0;
        
        let chart_width = label_width + (self.days_to_show as f32 * day_width);
        let chart_height = filtered_tasks.len() as f32 * row_height + 50.0;
        
        let (response, painter) = ui.allocate_painter(
            Vec2::new(chart_width, chart_height),
            Sense::click_and_drag(),
        );
        
        let rect = response.rect;
        
        // Draw background
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
        
        // Draw header with dates
        for day in 0..self.days_to_show {
            let date = self.start_date + Duration::days(day);
            let x = rect.min.x + label_width + (day as f32 * day_width);
            
            // Draw date text
            painter.text(
                Pos2::new(x + day_width / 2.0, rect.min.y + 10.0),
                egui::Align2::CENTER_CENTER,
                format!("{}/{}", date.month(), date.day()),
                egui::FontId::proportional(10.0),
                ui.visuals().text_color(),
            );
            
            // Draw vertical grid line
            painter.line_segment(
                [Pos2::new(x, rect.min.y + 20.0), Pos2::new(x, rect.max.y)],
                Stroke::new(0.5, ui.visuals().widgets.noninteractive.bg_stroke.color),
            );
        }
        
        // Draw tasks
        for (index, (task_id, task)) in filtered_tasks.iter().enumerate() {
            let y = rect.min.y + 30.0 + (index as f32 * row_height);
            
            // Draw task label
            let label_color = if Some(*task_id) == self.selected_task_id {
                ui.visuals().strong_text_color()
            } else {
                ui.visuals().text_color()
            };
            
            painter.text(
                Pos2::new(rect.min.x + 5.0, y + row_height / 2.0),
                egui::Align2::LEFT_CENTER,
                &task.title,
                egui::FontId::proportional(12.0),
                label_color,
            );
            
            // Draw task bar if dates are set
            if let (Some(start), Some(end)) = (task.scheduled_date, task.due_date) {
                let start_days = (start.date_naive() - self.start_date).num_days();
                let duration = (end.date_naive() - start.date_naive()).num_days() + 1;
                
                if start_days < self.days_to_show && start_days + duration > 0 {
                    let bar_x = rect.min.x + label_width + (start_days.max(0) as f32 * day_width);
                    let bar_width = (duration.min(self.days_to_show - start_days) as f32 * day_width).max(day_width);
                    
                    let bar_color = match task.status {
                        crate::domain::task::TaskStatus::Done => Color32::from_rgb(76, 175, 80),
                        crate::domain::task::TaskStatus::InProgress => Color32::from_rgb(33, 150, 243),
                        crate::domain::task::TaskStatus::Blocked => Color32::from_rgb(244, 67, 54),
                        crate::domain::task::TaskStatus::Review => Color32::from_rgb(255, 193, 7),
                        crate::domain::task::TaskStatus::Todo => Color32::from_rgb(158, 158, 158),
                        crate::domain::task::TaskStatus::Cancelled => Color32::from_rgb(128, 128, 128),
                    };
                    
                    let bar_rect = Rect::from_min_size(
                        Pos2::new(bar_x, y + 5.0),
                        Vec2::new(bar_width, row_height - 10.0),
                    );
                    
                    painter.rect_filled(bar_rect, 2.0, bar_color);
                    
                    // Handle click on bar
                    if response.clicked() && bar_rect.contains(response.interact_pointer_pos().unwrap_or_default()) {
                        self.selected_task_id = Some(*task_id);
                    }
                }
            }
            
            // Draw horizontal grid line
            painter.line_segment(
                [Pos2::new(rect.min.x, y + row_height), Pos2::new(rect.max.x, y + row_height)],
                Stroke::new(0.5, ui.visuals().widgets.noninteractive.bg_stroke.color),
            );
        }
    }
    
    fn show_list_view(&mut self, ui: &mut Ui, tasks: &[Task]) {
        let task_map: HashMap<Uuid, Task> = tasks.iter()
            .map(|t| (t.id, t.clone()))
            .collect();
        
        let filtered_tasks = self.apply_filters(&task_map);
        
        // Simple list view
        for task in filtered_tasks.values() {
            ui.horizontal(|ui| {
                // Status icon
                let status_icon = match task.status {
                    crate::domain::task::TaskStatus::Todo => "⭕",
                    crate::domain::task::TaskStatus::InProgress => "🔄",
                    crate::domain::task::TaskStatus::Done => "✅",
                    crate::domain::task::TaskStatus::Blocked => "🚫",
                    crate::domain::task::TaskStatus::Review => "👁",
                    crate::domain::task::TaskStatus::Cancelled => "❌",
                };
                ui.label(status_icon);
                
                // Task title
                if ui.selectable_label(
                    Some(task.id) == self.selected_task_id,
                    &task.title
                ).clicked() {
                    self.selected_task_id = Some(task.id);
                }
                
                // Due date
                if let Some(due) = task.due_date {
                    let days_until = (due.date_naive() - Local::now().naive_local().date()).num_days();
                    let date_text = if days_until < 0 {
                        format!("⚠️ {} days overdue", -days_until)
                    } else if days_until == 0 {
                        "📅 Due today".to_string()
                    } else {
                        format!("📅 {} days", days_until)
                    };
                    ui.label(date_text);
                }
            });
        }
    }
    
    fn show_calendar_view(&mut self, ui: &mut Ui, tasks: &[Task]) {
        ui.label("Calendar view coming soon...");
        // TODO: Implement calendar view
    }
}