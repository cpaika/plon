use crate::domain::{task::Task, resource::Resource, dependency::Dependency};
use crate::ui::widgets::gantt_chart::GanttChart;
use chrono::{NaiveDate, Duration, Local, Datelike};
use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, RichText, FontId};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct GanttView {
    gantt_chart: GanttChart,
    scroll_offset: f32,
    selected_task_id: Option<Uuid>,
    show_grid: bool,
    show_today_line: bool,
    show_weekends: bool,
    filter_by_resource: Option<Uuid>,
    collapsed_groups: HashSet<String>,
    row_height: f32,
    column_width: f32,
    header_height: f32,
}

impl GanttView {
    pub fn new() -> Self {
        Self {
            gantt_chart: GanttChart::new(),
            scroll_offset: 0.0,
            selected_task_id: None,
            show_grid: true,
            show_today_line: true,
            show_weekends: true,
            filter_by_resource: None,
            collapsed_groups: HashSet::new(),
            row_height: 30.0,
            column_width: 30.0,
            header_height: 60.0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &[Task], resources: &[Resource], dependencies: &[Dependency]) {
        // Top toolbar
        ui.horizontal(|ui| {
            ui.heading("📊 Gantt Chart");
            
            ui.separator();
            
            // Zoom controls
            if ui.button("🔍+").clicked() {
                self.gantt_chart.zoom_in();
                self.column_width = (self.column_width * 1.2).min(100.0);
            }
            
            if ui.button("🔍-").clicked() {
                self.gantt_chart.zoom_out();
                self.column_width = (self.column_width / 1.2).max(10.0);
            }
            
            if ui.button("🔍 Reset").clicked() {
                self.gantt_chart.zoom_level = 1.0;
                self.column_width = 30.0;
            }
            
            ui.separator();
            
            // View options
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_today_line, "Today");
            ui.checkbox(&mut self.show_weekends, "Weekends");
            ui.checkbox(&mut self.gantt_chart.show_dependencies, "Dependencies");
            ui.checkbox(&mut self.gantt_chart.show_resources, "Resources");
            
            ui.separator();
            
            // Resource filter
            ui.label("Filter by:");
            egui::ComboBox::from_label("")
                .selected_text(
                    if let Some(resource_id) = self.filter_by_resource {
                        resources.iter()
                            .find(|r| r.id == resource_id)
                            .map(|r| r.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string())
                    } else {
                        "All Resources".to_string()
                    }
                )
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.filter_by_resource, None, "All Resources").clicked() {
                        self.filter_by_resource = None;
                    }
                    
                    for resource in resources {
                        if ui.selectable_value(
                            &mut self.filter_by_resource, 
                            Some(resource.id), 
                            &resource.name
                        ).clicked() {
                            self.filter_by_resource = Some(resource.id);
                        }
                    }
                });
        });
        
        ui.separator();
        
        // Main Gantt chart area
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_gantt(ui, tasks, resources, dependencies);
            });
    }
    
    fn render_gantt(&mut self, ui: &mut Ui, tasks: &[Task], resources: &[Resource], dependencies: &[Dependency]) {
        let start_date = self.gantt_chart.get_start_date();
        let days_to_show = self.gantt_chart.days_to_show;
        
        // Filter tasks
        let filtered_tasks: Vec<&Task> = tasks.iter()
            .filter(|task| {
                if let Some(resource_id) = self.filter_by_resource {
                    task.assigned_resource_id == Some(resource_id)
                } else {
                    true
                }
            })
            .collect();
        
        // Calculate dimensions
        let chart_width = self.column_width * days_to_show as f32;
        let chart_height = self.row_height * filtered_tasks.len() as f32 + self.header_height;
        
        let (response, painter) = ui.allocate_painter(
            Vec2::new(chart_width + 300.0, chart_height),
            Sense::click_and_drag(),
        );
        
        let rect = response.rect;
        
        // Draw background
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
        
        // Draw header (dates)
        self.draw_header(&painter, rect, start_date, days_to_show, ui);
        
        // Draw grid
        if self.show_grid {
            self.draw_grid(&painter, rect, days_to_show, filtered_tasks.len(), ui);
        }
        
        // Draw weekends
        if self.show_weekends {
            self.draw_weekends(&painter, rect, start_date, days_to_show, ui);
        }
        
        // Draw today line
        if self.show_today_line {
            self.draw_today_line(&painter, rect, start_date, ui);
        }
        
        // Draw tasks
        for (index, task) in filtered_tasks.iter().enumerate() {
            self.draw_task_row(&painter, rect, task, index, start_date, resources, ui);
        }
        
        // Draw dependencies
        if self.gantt_chart.show_dependencies {
            self.draw_dependencies(&painter, rect, &filtered_tasks, dependencies, start_date, ui);
        }
        
        // Handle interactions
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.handle_click(pos, rect, &filtered_tasks);
            }
        }
    }
    
    fn draw_header(&self, painter: &egui::Painter, rect: Rect, start_date: NaiveDate, days: i64, ui: &Ui) {
        let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), self.header_height));
        
        // Draw header background
        painter.rect_filled(header_rect, 0.0, ui.visuals().widgets.noninteractive.bg_fill);
        
        // Draw month labels
        let mut current_month = start_date.month();
        let mut month_start_x = 300.0;
        
        for day_offset in 0..days {
            let date = start_date + Duration::days(day_offset);
            let x = rect.min.x + 300.0 + (day_offset as f32 * self.column_width);
            
            // Draw month separator and label
            if date.month() != current_month || day_offset == 0 {
                if day_offset > 0 {
                    painter.line_segment(
                        [Pos2::new(x, rect.min.y), Pos2::new(x, rect.min.y + 30.0)],
                        Stroke::new(1.0, ui.visuals().widgets.noninteractive.fg_stroke.color),
                    );
                }
                
                let month_name = match date.month() {
                    1 => "January",
                    2 => "February",
                    3 => "March",
                    4 => "April",
                    5 => "May",
                    6 => "June",
                    7 => "July",
                    8 => "August",
                    9 => "September",
                    10 => "October",
                    11 => "November",
                    12 => "December",
                    _ => "",
                };
                
                painter.text(
                    Pos2::new(x + 5.0, rect.min.y + 10.0),
                    egui::Align2::LEFT_CENTER,
                    format!("{} {}", month_name, date.year()),
                    FontId::proportional(12.0),
                    ui.visuals().text_color(),
                );
                
                current_month = date.month();
                month_start_x = x;
            }
            
            // Draw day number
            painter.text(
                Pos2::new(x + self.column_width / 2.0, rect.min.y + 40.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", date.day()),
                FontId::proportional(10.0),
                ui.visuals().text_color(),
            );
        }
    }
    
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, days: i64, task_count: usize, ui: &Ui) {
        let grid_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        
        // Vertical lines (days)
        for day in 0..=days {
            let x = rect.min.x + 300.0 + (day as f32 * self.column_width);
            painter.line_segment(
                [Pos2::new(x, rect.min.y + self.header_height), Pos2::new(x, rect.max.y)],
                Stroke::new(0.5, grid_color),
            );
        }
        
        // Horizontal lines (tasks)
        for row in 0..=task_count {
            let y = rect.min.y + self.header_height + (row as f32 * self.row_height);
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(0.5, grid_color),
            );
        }
    }
    
    fn draw_weekends(&self, painter: &egui::Painter, rect: Rect, start_date: NaiveDate, days: i64, ui: &Ui) {
        for day_offset in 0..days {
            let date = start_date + Duration::days(day_offset);
            
            // Check if weekend (Saturday = 6, Sunday = 0 in weekday())
            if date.weekday().num_days_from_monday() >= 5 {
                let x = rect.min.x + 300.0 + (day_offset as f32 * self.column_width);
                let weekend_rect = Rect::from_min_size(
                    Pos2::new(x, rect.min.y + self.header_height),
                    Vec2::new(self.column_width, rect.height() - self.header_height),
                );
                
                painter.rect_filled(
                    weekend_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(128, 128, 128, 20),
                );
            }
        }
    }
    
    fn draw_today_line(&self, painter: &egui::Painter, rect: Rect, start_date: NaiveDate, ui: &Ui) {
        let today = Local::now().naive_local().date();
        let days_from_start = (today - start_date).num_days();
        
        if days_from_start >= 0 && days_from_start < self.gantt_chart.days_to_show {
            let x = rect.min.x + 300.0 + (days_from_start as f32 * self.column_width) + self.column_width / 2.0;
            
            painter.line_segment(
                [Pos2::new(x, rect.min.y + self.header_height), Pos2::new(x, rect.max.y)],
                Stroke::new(2.0, Color32::from_rgb(255, 0, 0)),
            );
            
            // Today label
            painter.text(
                Pos2::new(x, rect.min.y + self.header_height - 5.0),
                egui::Align2::CENTER_BOTTOM,
                "Today",
                FontId::proportional(10.0),
                Color32::from_rgb(255, 0, 0),
            );
        }
    }
    
    fn draw_task_row(&self, painter: &egui::Painter, rect: Rect, task: &Task, row: usize, start_date: NaiveDate, resources: &[Resource], ui: &Ui) {
        let y = rect.min.y + self.header_height + (row as f32 * self.row_height);
        
        // Draw task name
        let task_name_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, y),
            Vec2::new(300.0, self.row_height),
        );
        
        painter.rect_filled(
            task_name_rect,
            0.0,
            ui.visuals().widgets.noninteractive.bg_fill,
        );
        
        // Task title with status icon
        let status_icon = match task.status {
            crate::domain::task::TaskStatus::Todo => "⭕",
            crate::domain::task::TaskStatus::InProgress => "🔄",
            crate::domain::task::TaskStatus::Done => "✅",
            crate::domain::task::TaskStatus::Blocked => "🚫",
            crate::domain::task::TaskStatus::Review => "👁",
            crate::domain::task::TaskStatus::Cancelled => "❌",
        };
        
        painter.text(
            Pos2::new(rect.min.x + 5.0, y + self.row_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{} {}", status_icon, task.title),
            FontId::proportional(12.0),
            ui.visuals().text_color(),
        );
        
        // Draw resource name if assigned
        if let Some(resource_id) = task.assigned_resource_id {
            if let Some(resource) = resources.iter().find(|r| r.id == resource_id) {
                painter.text(
                    Pos2::new(rect.min.x + 200.0, y + self.row_height / 2.0),
                    egui::Align2::LEFT_CENTER,
                    &resource.name,
                    FontId::proportional(10.0),
                    ui.visuals().weak_text_color(),
                );
            }
        }
        
        // Draw task bar
        if let (Some(start), Some(end)) = (task.scheduled_date, task.due_date) {
            let start_date_naive = start.naive_local().date();
            let end_date_naive = end.naive_local().date();
            
            let days_from_chart_start = (start_date_naive - start_date).num_days();
            let duration = (end_date_naive - start_date_naive).num_days() + 1;
            
            if days_from_chart_start < self.gantt_chart.days_to_show && days_from_chart_start + duration > 0 {
                let bar_start = days_from_chart_start.max(0);
                let bar_end = (days_from_chart_start + duration).min(self.gantt_chart.days_to_show);
                let bar_duration = bar_end - bar_start;
                
                let x = rect.min.x + 300.0 + (bar_start as f32 * self.column_width);
                let width = bar_duration as f32 * self.column_width;
                
                let bar_rect = Rect::from_min_size(
                    Pos2::new(x, y + 5.0),
                    Vec2::new(width, self.row_height - 10.0),
                );
                
                // Choose color based on status
                let color = match task.status {
                    crate::domain::task::TaskStatus::Done => Color32::from_rgb(76, 175, 80),
                    crate::domain::task::TaskStatus::InProgress => Color32::from_rgb(33, 150, 243),
                    crate::domain::task::TaskStatus::Blocked => Color32::from_rgb(244, 67, 54),
                    crate::domain::task::TaskStatus::Review => Color32::from_rgb(255, 193, 7),
                    crate::domain::task::TaskStatus::Todo => Color32::from_rgb(158, 158, 158),
                    crate::domain::task::TaskStatus::Cancelled => Color32::from_rgb(128, 128, 128),
                };
                
                // Draw bar
                painter.rect_filled(bar_rect, 2.0, color);
                
                // Draw progress if in progress
                if task.status == crate::domain::task::TaskStatus::InProgress && !task.subtasks.is_empty() {
                    let (completed, total) = task.subtask_progress();
                    let progress = completed as f32 / total as f32;
                    
                    let progress_rect = Rect::from_min_size(
                        bar_rect.min,
                        Vec2::new(bar_rect.width() * progress, bar_rect.height()),
                    );
                    
                    painter.rect_filled(progress_rect, 2.0, color.gamma_multiply(1.3));
                }
                
                // Draw border if selected
                if Some(task.id) == self.selected_task_id {
                    painter.rect_stroke(bar_rect, 2.0, Stroke::new(2.0, Color32::WHITE));
                }
            }
        }
    }
    
    fn draw_dependencies(&self, painter: &egui::Painter, rect: Rect, tasks: &[&Task], dependencies: &[Dependency], start_date: NaiveDate, ui: &Ui) {
        // Create task position map
        let mut task_positions: HashMap<Uuid, (usize, Option<NaiveDate>, Option<NaiveDate>)> = HashMap::new();
        
        for (index, task) in tasks.iter().enumerate() {
            let start = task.scheduled_date.map(|d| d.naive_local().date());
            let end = task.due_date.map(|d| d.naive_local().date());
            task_positions.insert(task.id, (index, start, end));
        }
        
        // Draw dependency lines
        for dep in dependencies {
            if let (Some(&(from_row, from_start, from_end)), Some(&(to_row, to_start, to_end))) = 
                (task_positions.get(&dep.from_task_id), task_positions.get(&dep.to_task_id)) {
                
                if let (Some(from_end_date), Some(to_start_date)) = (from_end, to_start) {
                    let from_days = (from_end_date - start_date).num_days();
                    let to_days = (to_start_date - start_date).num_days();
                    
                    if from_days >= 0 && from_days < self.gantt_chart.days_to_show &&
                       to_days >= 0 && to_days < self.gantt_chart.days_to_show {
                        
                        let from_x = rect.min.x + 300.0 + ((from_days + 1) as f32 * self.column_width);
                        let from_y = rect.min.y + self.header_height + (from_row as f32 * self.row_height) + self.row_height / 2.0;
                        
                        let to_x = rect.min.x + 300.0 + (to_days as f32 * self.column_width);
                        let to_y = rect.min.y + self.header_height + (to_row as f32 * self.row_height) + self.row_height / 2.0;
                        
                        // Draw arrow line
                        painter.line_segment(
                            [Pos2::new(from_x, from_y), Pos2::new(to_x, to_y)],
                            Stroke::new(1.0, ui.visuals().weak_text_color()),
                        );
                        
                        // Draw arrowhead
                        let angle = (to_y - from_y).atan2(to_x - from_x);
                        let arrow_size = 5.0;
                        
                        painter.line_segment(
                            [
                                Pos2::new(to_x, to_y),
                                Pos2::new(
                                    to_x - arrow_size * angle.cos() - arrow_size * angle.sin(),
                                    to_y - arrow_size * angle.sin() + arrow_size * angle.cos()
                                ),
                            ],
                            Stroke::new(1.0, ui.visuals().weak_text_color()),
                        );
                        
                        painter.line_segment(
                            [
                                Pos2::new(to_x, to_y),
                                Pos2::new(
                                    to_x - arrow_size * angle.cos() + arrow_size * angle.sin(),
                                    to_y - arrow_size * angle.sin() - arrow_size * angle.cos()
                                ),
                            ],
                            Stroke::new(1.0, ui.visuals().weak_text_color()),
                        );
                    }
                }
            }
        }
    }
    
    fn handle_click(&mut self, pos: Pos2, rect: Rect, tasks: &[&Task]) {
        // Check if click is in task area
        if pos.x > rect.min.x + 300.0 && pos.y > rect.min.y + self.header_height {
            let row = ((pos.y - rect.min.y - self.header_height) / self.row_height) as usize;
            
            if row < tasks.len() {
                self.selected_task_id = Some(tasks[row].id);
            }
        }
    }
    
    pub fn get_selected_task(&self) -> Option<Uuid> {
        self.selected_task_id
    }
}