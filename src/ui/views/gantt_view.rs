use crate::domain::{dependency::Dependency, resource::Resource, task::Task};
use crate::ui::widgets::gantt_chart::{DragOperation, GanttChart, InteractiveGanttChart};
use chrono::{Datelike, Duration, Local, NaiveDate, Utc};
use eframe::egui::{self, Color32, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct GanttView {
    gantt_chart: GanttChart,
    interactive_chart: InteractiveGanttChart,
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

impl Default for GanttView {
    fn default() -> Self {
        Self::new()
    }
}

impl GanttView {
    pub fn new() -> Self {
        Self {
            gantt_chart: GanttChart::new(),
            interactive_chart: InteractiveGanttChart::new(),
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

    pub fn show(
        &mut self,
        ui: &mut Ui,
        tasks: &mut [Task],
        resources: &[Resource],
        dependencies: &[Dependency],
    ) -> bool {
        let mut tasks_modified = false;
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
                .selected_text(if let Some(resource_id) = self.filter_by_resource {
                    resources
                        .iter()
                        .find(|r| r.id == resource_id)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "All Resources".to_string()
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut self.filter_by_resource, None, "All Resources")
                        .clicked()
                    {
                        self.filter_by_resource = None;
                    }

                    for resource in resources {
                        if ui
                            .selectable_value(
                                &mut self.filter_by_resource,
                                Some(resource.id),
                                &resource.name,
                            )
                            .clicked()
                        {
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
                tasks_modified = self.render_gantt(ui, tasks, resources, dependencies);
            });

        tasks_modified
    }

    fn render_gantt(
        &mut self,
        ui: &mut Ui,
        tasks: &mut [Task],
        resources: &[Resource],
        dependencies: &[Dependency],
    ) -> bool {
        let mut tasks_modified = false;
        let start_date = self.gantt_chart.get_start_date();
        let days_to_show = self.gantt_chart.days_to_show;

        // Filter tasks - collect IDs first to avoid borrow issues
        let filtered_task_ids: Vec<Uuid> = tasks
            .iter()
            .filter(|task| {
                if let Some(resource_id) = self.filter_by_resource {
                    task.assigned_resource_id == Some(resource_id)
                } else {
                    true
                }
            })
            .map(|task| task.id)
            .collect();

        // Create a temporary vec for display purposes
        let filtered_tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| filtered_task_ids.contains(&task.id))
            .cloned()
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

        // Track task rectangles for interaction
        let mut task_rects: HashMap<Uuid, Rect> = HashMap::new();

        // Draw tasks
        for (index, task) in filtered_tasks.iter().enumerate() {
            let task_rect =
                self.draw_task_row(&painter, rect, task, index, start_date, resources, ui);
            if let Some(bar_rect) = task_rect {
                task_rects.insert(task.id, bar_rect);
            }
        }

        // Draw dependencies
        if self.gantt_chart.show_dependencies {
            let filtered_task_refs: Vec<&Task> = filtered_tasks.iter().collect();
            self.draw_dependencies(
                &painter,
                rect,
                &filtered_task_refs,
                dependencies,
                start_date,
                ui,
            );
        }

        // Handle hover and cursor changes
        if let Some(hover_pos) = response.hover_pos() {
            let mut cursor_set = false;
            for (task_id, task_rect) in &task_rects {
                if self
                    .interactive_chart
                    .update_hover(hover_pos, *task_id, *task_rect)
                {
                    if let Some(cursor) = self
                        .interactive_chart
                        .get_hover_cursor(hover_pos, *task_rect)
                    {
                        ui.ctx().set_cursor_icon(cursor);
                        cursor_set = true;
                    }
                    break;
                }
            }
            if !cursor_set {
                ui.ctx().set_cursor_icon(CursorIcon::Default);
            }
        }

        // Handle drag operations
        if response.drag_started()
            && let Some(pos) = response.interact_pointer_pos()
        {
            // Find which task was clicked and where
            for (task_id, task_rect) in &task_rects {
                if task_rect.contains(pos) {
                    // Find the corresponding task
                    if let Some(task) = tasks.iter().find(|t| t.id == *task_id)
                        && let (Some(start_date_time), Some(end_date_time)) =
                            (task.scheduled_date, task.due_date)
                    {
                        let task_start = start_date_time.naive_local().date();
                        let task_end = end_date_time.naive_local().date();

                        // Determine drag operation type
                        if self.interactive_chart.is_near_left_handle(pos, *task_rect) {
                            self.interactive_chart
                                .start_drag(DragOperation::ResizeStart {
                                    task_id: *task_id,
                                    initial_start: task_start,
                                    initial_end: task_end,
                                    drag_start_pos: pos,
                                });
                        } else if self.interactive_chart.is_near_right_handle(pos, *task_rect) {
                            self.interactive_chart.start_drag(DragOperation::ResizeEnd {
                                task_id: *task_id,
                                initial_start: task_start,
                                initial_end: task_end,
                                drag_start_pos: pos,
                            });
                        } else {
                            self.interactive_chart
                                .start_drag(DragOperation::Reschedule {
                                    task_id: *task_id,
                                    initial_start: task_start,
                                    initial_end: task_end,
                                    drag_start_pos: pos,
                                });
                        }
                        break;
                    }
                }
            }
        }

        // Update drag position and draw preview
        if self.interactive_chart.is_dragging()
            && response.dragged()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let (preview_start, preview_end) =
                self.interactive_chart
                    .update_drag(pos, start_date, self.column_width);

            // Draw preview of new position
            if let Some(dragging_task_id) = self.interactive_chart.get_dragging_task_id() {
                // Find the task index
                if let Some(task_index) =
                    filtered_tasks.iter().position(|t| t.id == dragging_task_id)
                {
                    let preview_style = self.interactive_chart.get_drag_preview_style();
                    self.draw_task_preview(
                        &painter,
                        rect,
                        preview_start,
                        preview_end,
                        task_index,
                        preview_style,
                    );
                }
            }
        }

        // Complete drag operation
        if response.drag_stopped()
            && self.interactive_chart.is_dragging()
            && let Some(pos) = response.interact_pointer_pos()
            && let Some((task_id, new_start, new_end)) =
                self.interactive_chart
                    .complete_drag(pos, start_date, self.column_width)
        {
            // Update the task with new dates
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.scheduled_date = Some(chrono::DateTime::from_naive_utc_and_offset(
                    new_start.and_hms_opt(0, 0, 0).unwrap(),
                    Utc,
                ));
                task.due_date = Some(chrono::DateTime::from_naive_utc_and_offset(
                    new_end.and_hms_opt(23, 59, 59).unwrap(),
                    Utc,
                ));
                task.updated_at = Utc::now();
                tasks_modified = true;
            }
        }

        // Handle regular click (selection)
        if response.clicked()
            && !self.interactive_chart.is_dragging()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let filtered_task_refs: Vec<&Task> = filtered_tasks.iter().collect();
            self.handle_click(pos, rect, &filtered_task_refs);
        }

        // Cancel drag on escape
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) && self.interactive_chart.is_dragging() {
            self.interactive_chart.cancel_drag();
        }

        tasks_modified
    }

    fn draw_header(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        start_date: NaiveDate,
        days: i64,
        ui: &Ui,
    ) {
        let header_rect =
            Rect::from_min_size(rect.min, Vec2::new(rect.width(), self.header_height));

        // Draw header background
        painter.rect_filled(
            header_rect,
            0.0,
            ui.visuals().widgets.noninteractive.bg_fill,
        );

        // Draw month labels
        let mut current_month = start_date.month();
        let mut _month_start_x = 300.0;

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
                _month_start_x = x;
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

    fn draw_grid(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        days: i64,
        task_count: usize,
        ui: &Ui,
    ) {
        let grid_color = ui.visuals().widgets.noninteractive.bg_stroke.color;

        // Vertical lines (days)
        for day in 0..=days {
            let x = rect.min.x + 300.0 + (day as f32 * self.column_width);
            painter.line_segment(
                [
                    Pos2::new(x, rect.min.y + self.header_height),
                    Pos2::new(x, rect.max.y),
                ],
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

    fn draw_weekends(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        start_date: NaiveDate,
        days: i64,
        _ui: &Ui,
    ) {
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

    fn draw_today_line(&self, painter: &egui::Painter, rect: Rect, start_date: NaiveDate, _ui: &Ui) {
        let today = Local::now().naive_local().date();
        let days_from_start = (today - start_date).num_days();

        if days_from_start >= 0 && days_from_start < self.gantt_chart.days_to_show {
            let x = rect.min.x
                + 300.0
                + (days_from_start as f32 * self.column_width)
                + self.column_width / 2.0;

            painter.line_segment(
                [
                    Pos2::new(x, rect.min.y + self.header_height),
                    Pos2::new(x, rect.max.y),
                ],
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

    fn draw_task_row(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        task: &Task,
        row: usize,
        start_date: NaiveDate,
        resources: &[Resource],
        ui: &Ui,
    ) -> Option<Rect> {
        let y = rect.min.y + self.header_height + (row as f32 * self.row_height);

        // Draw task name
        let task_name_rect =
            Rect::from_min_size(Pos2::new(rect.min.x, y), Vec2::new(300.0, self.row_height));

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
        if let Some(resource_id) = task.assigned_resource_id
            && let Some(resource) = resources.iter().find(|r| r.id == resource_id)
        {
            painter.text(
                Pos2::new(rect.min.x + 200.0, y + self.row_height / 2.0),
                egui::Align2::LEFT_CENTER,
                &resource.name,
                FontId::proportional(10.0),
                ui.visuals().weak_text_color(),
            );
        }

        // Draw task bar
        if let (Some(start), Some(end)) = (task.scheduled_date, task.due_date) {
            let start_date_naive = start.naive_local().date();
            let end_date_naive = end.naive_local().date();

            let days_from_chart_start = (start_date_naive - start_date).num_days();
            let duration = (end_date_naive - start_date_naive).num_days() + 1;

            if days_from_chart_start < self.gantt_chart.days_to_show
                && days_from_chart_start + duration > 0
            {
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
                if task.status == crate::domain::task::TaskStatus::InProgress
                    && !task.subtasks.is_empty()
                {
                    let (completed, total) = task.subtask_progress();
                    let progress = completed as f32 / total as f32;

                    let progress_rect = Rect::from_min_size(
                        bar_rect.min,
                        Vec2::new(bar_rect.width() * progress, bar_rect.height()),
                    );

                    painter.rect_filled(progress_rect, 2.0, color.gamma_multiply(1.3));
                }

                // Draw border if selected or hovering
                if Some(task.id) == self.selected_task_id {
                    painter.rect_stroke(bar_rect, 2.0, Stroke::new(2.0, Color32::WHITE));
                } else if Some(task.id) == self.interactive_chart.hovered_task_id {
                    painter.rect_stroke(
                        bar_rect,
                        2.0,
                        Stroke::new(1.0, Color32::from_rgb(100, 100, 255)),
                    );
                }

                // Draw resize handles when hovering
                if Some(task.id) == self.interactive_chart.hovered_task_id {
                    // Left handle
                    painter.circle_filled(
                        Pos2::new(bar_rect.min.x, bar_rect.center().y),
                        3.0,
                        Color32::WHITE,
                    );
                    // Right handle
                    painter.circle_filled(
                        Pos2::new(bar_rect.max.x, bar_rect.center().y),
                        3.0,
                        Color32::WHITE,
                    );
                }

                return Some(bar_rect);
            }
        }
        None
    }

    fn draw_task_preview(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        start: NaiveDate,
        end: NaiveDate,
        row: usize,
        style: crate::ui::widgets::gantt_chart::DragPreviewStyle,
    ) {
        let y = rect.min.y + self.header_height + (row as f32 * self.row_height);
        let chart_start = self.gantt_chart.get_start_date();

        let days_from_chart_start = (start - chart_start).num_days();
        let duration = (end - start).num_days() + 1;

        if days_from_chart_start < self.gantt_chart.days_to_show
            && days_from_chart_start + duration > 0
        {
            let bar_start = days_from_chart_start.max(0);
            let bar_end = (days_from_chart_start + duration).min(self.gantt_chart.days_to_show);
            let bar_duration = bar_end - bar_start;

            let x = rect.min.x + 300.0 + (bar_start as f32 * self.column_width);
            let width = bar_duration as f32 * self.column_width;

            let preview_rect = Rect::from_min_size(
                Pos2::new(x, y + 5.0),
                Vec2::new(width, self.row_height - 10.0),
            );

            // Draw semi-transparent preview
            painter.rect(
                preview_rect,
                2.0,
                style.fill_color,
                Stroke::new(2.0, style.stroke_color),
            );
        }
    }

    fn draw_dependencies(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        tasks: &[&Task],
        dependencies: &[Dependency],
        start_date: NaiveDate,
        ui: &Ui,
    ) {
        // Create task position map
        let mut task_positions: HashMap<Uuid, (usize, Option<NaiveDate>, Option<NaiveDate>)> =
            HashMap::new();

        for (index, task) in tasks.iter().enumerate() {
            let start = task.scheduled_date.map(|d| d.naive_local().date());
            let end = task.due_date.map(|d| d.naive_local().date());
            task_positions.insert(task.id, (index, start, end));
        }

        // Draw dependency lines
        for dep in dependencies {
            if let (Some(&(from_row, _from_start, from_end)), Some(&(to_row, to_start, _to_end))) = (
                task_positions.get(&dep.from_task_id),
                task_positions.get(&dep.to_task_id),
            ) && let (Some(from_end_date), Some(to_start_date)) = (from_end, to_start)
            {
                let from_days = (from_end_date - start_date).num_days();
                let to_days = (to_start_date - start_date).num_days();

                if from_days >= 0
                    && from_days < self.gantt_chart.days_to_show
                    && to_days >= 0
                    && to_days < self.gantt_chart.days_to_show
                {
                    let from_x = rect.min.x + 300.0 + ((from_days + 1) as f32 * self.column_width);
                    let from_y = rect.min.y
                        + self.header_height
                        + (from_row as f32 * self.row_height)
                        + self.row_height / 2.0;

                    let to_x = rect.min.x + 300.0 + (to_days as f32 * self.column_width);
                    let to_y = rect.min.y
                        + self.header_height
                        + (to_row as f32 * self.row_height)
                        + self.row_height / 2.0;

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
                                to_y - arrow_size * angle.sin() + arrow_size * angle.cos(),
                            ),
                        ],
                        Stroke::new(1.0, ui.visuals().weak_text_color()),
                    );

                    painter.line_segment(
                        [
                            Pos2::new(to_x, to_y),
                            Pos2::new(
                                to_x - arrow_size * angle.cos() + arrow_size * angle.sin(),
                                to_y - arrow_size * angle.sin() - arrow_size * angle.cos(),
                            ),
                        ],
                        Stroke::new(1.0, ui.visuals().weak_text_color()),
                    );
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
