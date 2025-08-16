use crate::domain::{task::Task, resource::Resource};
use crate::services::timeline_scheduler::{TaskSchedule, TimelineSchedule};
use chrono::{NaiveDate, Datelike, Local, Duration};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2, CursorIcon};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GanttColor {
    Green,
    Yellow,
    Red,
    Blue,
    Gray,
}

impl GanttColor {
    pub fn to_color32(&self) -> Color32 {
        match self {
            GanttColor::Green => Color32::from_rgb(76, 175, 80),
            GanttColor::Yellow => Color32::from_rgb(255, 193, 7),
            GanttColor::Red => Color32::from_rgb(244, 67, 54),
            GanttColor::Blue => Color32::from_rgb(33, 150, 243),
            GanttColor::Gray => Color32::from_rgb(158, 158, 158),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub date: NaiveDate,
    pub color: GanttColor,
}

#[derive(Debug, Clone)]
pub struct GanttBar {
    pub x: f32,
    pub width: f32,
    pub duration_days: i64,
}

#[derive(Debug, Clone)]
pub struct DependencyLine {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
}

#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub percentage: f32,
    pub color: GanttColor,
}

#[derive(Debug, Clone)]
pub struct WeekendPosition {
    pub x: f32,
    pub width: f32,
    pub day_of_week: u32,
}

pub struct GanttChart {
    pub zoom_level: f32,
    pub days_to_show: i64,
    pub show_dependencies: bool,
    pub show_resources: bool,
    start_date: NaiveDate,
    critical_path: HashSet<Uuid>,
    milestones: Vec<Milestone>,
}

#[derive(Debug, Clone)]
pub enum DragOperation {
    Reschedule {
        task_id: Uuid,
        initial_start: NaiveDate,
        initial_end: NaiveDate,
        drag_start_pos: Pos2,
    },
    ResizeStart {
        task_id: Uuid,
        initial_start: NaiveDate,
        initial_end: NaiveDate,
        drag_start_pos: Pos2,
    },
    ResizeEnd {
        task_id: Uuid,
        initial_start: NaiveDate,
        initial_end: NaiveDate,
        drag_start_pos: Pos2,
    },
}

#[derive(Debug, Clone)]
pub struct DragPreviewStyle {
    pub opacity: f32,
    pub stroke_color: Color32,
    pub fill_color: Color32,
}

pub struct InteractiveGanttChart {
    pub current_drag_operation: Option<DragOperation>,
    pub hovered_task_id: Option<Uuid>,
    pub selected_task_id: Option<Uuid>,
    selected_tasks: HashSet<Uuid>,
    min_date: Option<NaiveDate>,
    max_date: Option<NaiveDate>,
    snap_to_grid: bool,
    handle_detection_threshold: f32,
}

impl InteractiveGanttChart {
    pub fn new() -> Self {
        Self {
            current_drag_operation: None,
            hovered_task_id: None,
            selected_task_id: None,
            selected_tasks: HashSet::new(),
            min_date: None,
            max_date: None,
            snap_to_grid: true,
            handle_detection_threshold: 5.0,
        }
    }
    
    pub fn start_drag(&mut self, operation: DragOperation) {
        self.current_drag_operation = Some(operation);
    }
    
    pub fn update_drag(&mut self, current_pos: Pos2, chart_start: NaiveDate, column_width: f32) -> (NaiveDate, NaiveDate) {
        if let Some(ref operation) = self.current_drag_operation {
            match operation {
                DragOperation::Reschedule { initial_start, initial_end, drag_start_pos, .. } => {
                    let delta_pixels = current_pos.x - drag_start_pos.x;
                    let delta_days = (delta_pixels / column_width).round() as i64;
                    
                    let new_start = *initial_start + Duration::days(delta_days);
                    let new_end = *initial_end + Duration::days(delta_days);
                    
                    (self.constrain_date(new_start), self.constrain_date(new_end))
                }
                DragOperation::ResizeStart { initial_start, initial_end, drag_start_pos, .. } => {
                    let delta_pixels = current_pos.x - drag_start_pos.x;
                    let delta_days = (delta_pixels / column_width).round() as i64;
                    
                    let new_start = *initial_start + Duration::days(delta_days);
                    let new_start = new_start.min(*initial_end); // Don't go past end
                    
                    (self.constrain_date(new_start), *initial_end)
                }
                DragOperation::ResizeEnd { initial_start, initial_end, drag_start_pos, .. } => {
                    let delta_pixels = current_pos.x - drag_start_pos.x;
                    let delta_days = (delta_pixels / column_width).round() as i64;
                    
                    let new_end = *initial_end + Duration::days(delta_days);
                    let new_end = new_end.max(*initial_start); // Don't go before start
                    
                    (*initial_start, self.constrain_date(new_end))
                }
            }
        } else {
            (chart_start, chart_start)
        }
    }
    
    pub fn complete_drag(&mut self, final_pos: Pos2, chart_start: NaiveDate, column_width: f32) -> Option<(Uuid, NaiveDate, NaiveDate)> {
        if self.current_drag_operation.is_some() {
            let (new_start, new_end) = self.update_drag(final_pos, chart_start, column_width);
            let task_id = match self.current_drag_operation.as_ref().unwrap() {
                DragOperation::Reschedule { task_id, .. } |
                DragOperation::ResizeStart { task_id, .. } |
                DragOperation::ResizeEnd { task_id, .. } => *task_id,
            };
            
            self.current_drag_operation = None;
            Some((task_id, new_start, new_end))
        } else {
            None
        }
    }
    
    pub fn cancel_drag(&mut self) {
        self.current_drag_operation = None;
    }
    
    pub fn update_hover(&mut self, pos: Pos2, task_id: Uuid, task_rect: Rect) -> bool {
        if task_rect.contains(pos) {
            self.hovered_task_id = Some(task_id);
            true
        } else {
            if self.hovered_task_id == Some(task_id) {
                self.hovered_task_id = None;
            }
            false
        }
    }
    
    pub fn get_hover_cursor(&self, pos: Pos2, task_rect: Rect) -> Option<CursorIcon> {
        if self.is_near_left_handle(pos, task_rect) || self.is_near_right_handle(pos, task_rect) {
            Some(CursorIcon::ResizeHorizontal)
        } else if task_rect.contains(pos) {
            Some(CursorIcon::Move)
        } else {
            None
        }
    }
    
    pub fn is_near_left_handle(&self, pos: Pos2, rect: Rect) -> bool {
        if !rect.contains(pos) {
            return false;
        }
        (pos.x - rect.min.x).abs() <= self.handle_detection_threshold
    }
    
    pub fn is_near_right_handle(&self, pos: Pos2, rect: Rect) -> bool {
        if !rect.contains(pos) {
            return false;
        }
        (pos.x - rect.max.x).abs() <= self.handle_detection_threshold
    }
    
    pub fn is_dragging(&self) -> bool {
        self.current_drag_operation.is_some()
    }
    
    pub fn get_dragging_task_id(&self) -> Option<Uuid> {
        self.current_drag_operation.as_ref().map(|op| match op {
            DragOperation::Reschedule { task_id, .. } |
            DragOperation::ResizeStart { task_id, .. } |
            DragOperation::ResizeEnd { task_id, .. } => *task_id,
        })
    }
    
    pub fn get_drag_preview_style(&self) -> DragPreviewStyle {
        DragPreviewStyle {
            opacity: 0.6,
            stroke_color: Color32::from_rgb(0, 120, 215),
            fill_color: Color32::from_rgba_unmultiplied(0, 120, 215, 100),
        }
    }
    
    pub fn snap_to_grid(&self, pos: Pos2, column_width: f32) -> Pos2 {
        if self.snap_to_grid {
            let snapped_x = ((pos.x / column_width).round() * column_width);
            Pos2::new(snapped_x, pos.y)
        } else {
            pos
        }
    }
    
    pub fn set_min_date(&mut self, date: NaiveDate) {
        self.min_date = Some(date);
    }
    
    pub fn set_max_date(&mut self, date: NaiveDate) {
        self.max_date = Some(date);
    }
    
    pub fn constrain_date(&self, date: NaiveDate) -> NaiveDate {
        let mut constrained = date;
        
        if let Some(min) = self.min_date {
            constrained = constrained.max(min);
        }
        
        if let Some(max) = self.max_date {
            constrained = constrained.min(max);
        }
        
        constrained
    }
    
    pub fn select_task(&mut self, task_id: Uuid, add_to_selection: bool) {
        if add_to_selection {
            self.selected_tasks.insert(task_id);
        } else {
            self.selected_tasks.clear();
            self.selected_tasks.insert(task_id);
        }
        self.selected_task_id = Some(task_id);
    }
    
    pub fn selected_tasks(&self) -> Vec<Uuid> {
        self.selected_tasks.iter().copied().collect()
    }
    
    pub fn batch_reschedule(&self, offset_days: i64) -> Vec<(Uuid, i64)> {
        self.selected_tasks
            .iter()
            .map(|&task_id| (task_id, offset_days))
            .collect()
    }
}

impl Default for GanttChart {
    fn default() -> Self {
        Self::new()
    }
}

impl GanttChart {
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            days_to_show: 30,
            show_dependencies: true,
            show_resources: true,
            start_date: Local::now().naive_local().date(),
            critical_path: HashSet::new(),
            milestones: Vec::new(),
        }
    }
    
    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(3.0);
    }
    
    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.3);
    }
    
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
    }
    
    pub fn set_start_date(&mut self, date: NaiveDate) {
        self.start_date = date;
    }
    
    pub fn get_start_date(&self) -> NaiveDate {
        self.start_date
    }
    
    pub fn get_end_date(&self) -> NaiveDate {
        self.start_date + chrono::Duration::days(self.days_to_show - 1)
    }
    
    pub fn set_days_to_show(&mut self, days: i64) {
        self.days_to_show = days.max(7).min(365);
    }
    
    pub fn calculate_bar_position(&self, start: NaiveDate, end: NaiveDate, chart_width: f32) -> GanttBar {
        let chart_start = self.get_start_date();
        let chart_end = self.get_end_date();
        
        let days_from_start = (start - chart_start).num_days();
        let duration_days = (end - start).num_days() + 1;
        
        let day_width = chart_width / self.days_to_show as f32;
        let x = days_from_start as f32 * day_width * self.zoom_level;
        let width = duration_days as f32 * day_width * self.zoom_level;
        
        GanttBar {
            x,
            width,
            duration_days,
        }
    }
    
    pub fn group_tasks_by_resource(&self, tasks: &HashMap<Uuid, Task>) -> HashMap<Option<Uuid>, Vec<Uuid>> {
        let mut grouped = HashMap::new();
        
        for (task_id, task) in tasks {
            let resource_id = task.assigned_resource_id;
            grouped.entry(resource_id)
                .or_insert_with(Vec::new)
                .push(*task_id);
        }
        
        // Ensure we have an entry for unassigned tasks
        grouped.entry(None).or_insert_with(Vec::new);
        
        grouped
    }
    
    pub fn set_critical_path(&mut self, path: Vec<Uuid>) {
        self.critical_path = path.into_iter().collect();
    }
    
    pub fn is_on_critical_path(&self, task_id: Uuid) -> bool {
        self.critical_path.contains(&task_id)
    }
    
    pub fn calculate_dependency_line(
        &self,
        from_schedule: &TaskSchedule,
        to_schedule: &TaskSchedule,
        from_y: f32,
        to_y: f32,
        chart_width: f32,
    ) -> DependencyLine {
        let from_bar = self.calculate_bar_position(
            from_schedule.start_date,
            from_schedule.end_date,
            chart_width,
        );
        
        let to_bar = self.calculate_bar_position(
            to_schedule.start_date,
            to_schedule.end_date,
            chart_width,
        );
        
        DependencyLine {
            start_x: from_bar.x + from_bar.width,
            start_y: from_y,
            end_x: to_bar.x,
            end_y: to_y,
        }
    }
    
    pub fn add_milestone(&mut self, milestone: Milestone) {
        self.milestones.push(milestone);
    }
    
    pub fn get_milestones(&self) -> &[Milestone] {
        &self.milestones
    }
    
    pub fn calculate_today_line_position(&self, chart_width: f32) -> Option<f32> {
        let today = Local::now().naive_local().date();
        let chart_start = self.get_start_date();
        let chart_end = self.get_end_date();
        
        if today >= chart_start && today <= chart_end {
            let days_from_start = (today - chart_start).num_days();
            let day_width = chart_width / self.days_to_show as f32;
            Some(days_from_start as f32 * day_width * self.zoom_level)
        } else {
            None
        }
    }
    
    pub fn calculate_resource_utilization(&self, resource: &Resource) -> ResourceUtilization {
        let percentage = resource.utilization_percentage();
        let color = if percentage > 100.0 {
            GanttColor::Red
        } else if percentage > 80.0 {
            GanttColor::Yellow
        } else {
            GanttColor::Green
        };
        
        ResourceUtilization {
            percentage,
            color,
        }
    }
    
    pub fn get_weekend_positions(&self, chart_width: f32) -> Vec<WeekendPosition> {
        let mut weekends = Vec::new();
        let chart_start = self.get_start_date();
        let day_width = chart_width / self.days_to_show as f32;
        
        for day_offset in 0..self.days_to_show {
            let current_date = chart_start + chrono::Duration::days(day_offset);
            let weekday = current_date.weekday().num_days_from_monday();
            
            if weekday >= 5 { // Saturday or Sunday
                weekends.push(WeekendPosition {
                    x: day_offset as f32 * day_width * self.zoom_level,
                    width: day_width * self.zoom_level,
                    day_of_week: weekday,
                });
            }
        }
        
        weekends
    }
    
    pub fn export_to_json(
        &self,
        tasks: &HashMap<Uuid, Task>,
        resources: &HashMap<Uuid, Resource>,
        schedule: &TimelineSchedule,
    ) -> Result<String, String> {
        let export_data = serde_json::json!({
            "tasks": tasks.values().collect::<Vec<_>>(),
            "resources": resources.values().collect::<Vec<_>>(),
            "schedule": {
                "task_schedules": schedule.task_schedules.values().collect::<Vec<_>>(),
                "critical_path": schedule.critical_path,
                "warnings": schedule.warnings,
            },
            "chart_settings": {
                "start_date": self.start_date.to_string(),
                "days_to_show": self.days_to_show,
                "zoom_level": self.zoom_level,
            }
        });
        
        serde_json::to_string_pretty(&export_data)
            .map_err(|e| format!("Failed to export Gantt chart data: {}", e))
    }
    
    pub fn render(
        &mut self,
        ui: &mut Ui,
        tasks: &HashMap<Uuid, Task>,
        resources: &HashMap<Uuid, Resource>,
        schedule: &TimelineSchedule,
    ) {
        let available_size = ui.available_size();
        let chart_height = available_size.y;
        let chart_width = available_size.x - 200.0; // Reserve space for labels
        
        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Draw weekend highlights
                if self.show_resources {
                    let weekends = self.get_weekend_positions(chart_width);
                    for weekend in weekends {
                        let rect = Rect::from_min_size(
                            Pos2::new(weekend.x + 200.0, 0.0),
                            Vec2::new(weekend.width, chart_height),
                        );
                        ui.painter().rect_filled(
                            rect,
                            0.0,
                            Color32::from_rgba_unmultiplied(200, 200, 200, 30),
                        );
                    }
                }
                
                // Draw today line
                if let Some(today_x) = self.calculate_today_line_position(chart_width) {
                    ui.painter().line_segment(
                        [
                            Pos2::new(today_x + 200.0, 0.0),
                            Pos2::new(today_x + 200.0, chart_height),
                        ],
                        Stroke::new(2.0, Color32::from_rgb(255, 0, 0)),
                    );
                }
                
                // Group tasks by resource
                let grouped = self.group_tasks_by_resource(tasks);
                let mut y_position = 50.0;
                
                for (resource_id, task_ids) in grouped {
                    // Draw resource header
                    if let Some(resource_id) = resource_id {
                        if let Some(resource) = resources.get(&resource_id) {
                            ui.horizontal(|ui| {
                                ui.label(&resource.name);
                                let utilization = self.calculate_resource_utilization(resource);
                                ui.colored_label(
                                    utilization.color.to_color32(),
                                    format!("{}%", utilization.percentage.round()),
                                );
                            });
                            y_position += 30.0;
                        }
                    } else {
                        ui.label("Unassigned Tasks");
                        y_position += 30.0;
                    }
                    
                    // Draw task bars
                    for task_id in task_ids {
                        if let Some(task) = tasks.get(&task_id)
                            && let Some(task_schedule) = schedule.task_schedules.get(&task_id) {
                                let bar = self.calculate_bar_position(
                                    task_schedule.start_date,
                                    task_schedule.end_date,
                                    chart_width,
                                );
                                
                                let color = if self.is_on_critical_path(task_id) {
                                    Color32::from_rgb(255, 0, 0)
                                } else {
                                    Color32::from_rgb(33, 150, 243)
                                };
                                
                                let rect = Rect::from_min_size(
                                    Pos2::new(bar.x + 200.0, y_position),
                                    Vec2::new(bar.width, 25.0),
                                );
                                
                                let response = ui.allocate_rect(rect, Sense::hover());
                                ui.painter().rect_filled(rect, 2.0, color);
                                
                                // Draw task title
                                ui.painter().text(
                                    Pos2::new(10.0, y_position + 12.5),
                                    egui::Align2::LEFT_CENTER,
                                    &task.title,
                                    egui::FontId::default(),
                                    Color32::from_rgb(0, 0, 0),
                                );
                                
                                // Show tooltip on hover
                                if response.hovered() {
                                    egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("gantt_tooltip"), |ui| {
                                        ui.label(&task.title);
                                        ui.label(format!("Start: {}", task_schedule.start_date));
                                        ui.label(format!("End: {}", task_schedule.end_date));
                                        ui.label(format!("Duration: {} days", bar.duration_days));
                                        ui.label(format!("Hours: {}", task_schedule.allocated_hours));
                                    });
                                }
                                
                                y_position += 35.0;
                            }
                    }
                    
                    y_position += 20.0; // Space between resource groups
                }
                
                // Draw milestones
                for milestone in &self.milestones {
                    if let Some(x) = self.calculate_milestone_position(milestone.date, chart_width) {
                        ui.painter().circle_filled(
                            Pos2::new(x + 200.0, 25.0),
                            5.0,
                            milestone.color.to_color32(),
                        );
                    }
                }
            });
    }
    
    fn calculate_milestone_position(&self, date: NaiveDate, chart_width: f32) -> Option<f32> {
        let chart_start = self.get_start_date();
        let chart_end = self.get_end_date();
        
        if date >= chart_start && date <= chart_end {
            let days_from_start = (date - chart_start).num_days();
            let day_width = chart_width / self.days_to_show as f32;
            Some(days_from_start as f32 * day_width * self.zoom_level)
        } else {
            None
        }
    }
}

