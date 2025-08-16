use crate::domain::{task::Task, goal::Goal, resource::Resource, dependency::{Dependency, DependencyGraph, DependencyType}};
use crate::services::timeline_scheduler::{TimelineScheduler, TimelineSchedule};
use crate::ui::widgets::gantt_chart::GanttChart;
use eframe::egui::{self, Ui};
use chrono::{Utc, Duration, NaiveDate, Local};
use std::collections::HashMap;
use uuid::Uuid;

// Types are automatically exported when made public

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
    filter: TimelineFilter,
    gantt_chart: GanttChart,
    scheduler: TimelineScheduler,
    start_date: NaiveDate,
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
        }
    }
    
    pub fn set_view_mode(&mut self, mode: TimelineViewMode) {
        self.selected_view = mode;
    }
    
    pub fn set_filter(&mut self, filter: TimelineFilter) {
        self.filter = filter;
    }
    
    pub fn set_date_range(&mut self, days: i64) {
        self.days_to_show = days.max(7).min(365);
        self.gantt_chart.set_days_to_show(self.days_to_show);
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
        self.scheduler.calculate_schedule(tasks, resources, graph, self.start_date)
    }
    
    pub fn is_task_critical(&self, task_id: Uuid, critical_path: &[Uuid]) -> bool {
        critical_path.contains(&task_id)
    }
    
    pub fn export_timeline(
        &self,
        tasks: &HashMap<Uuid, Task>,
        resources: &HashMap<Uuid, Resource>,
        schedule: &TimelineSchedule,
    ) -> Result<String, String> {
        self.gantt_chart.export_to_json(tasks, resources, schedule)
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
        ui.heading("Timeline View");
        
        // View mode selector
        ui.horizontal(|ui| {
            ui.label("View Mode:");
            if ui.selectable_label(self.selected_view == TimelineViewMode::Gantt, "Gantt Chart").clicked() {
                self.set_view_mode(TimelineViewMode::Gantt);
            }
            if ui.selectable_label(self.selected_view == TimelineViewMode::List, "List").clicked() {
                self.set_view_mode(TimelineViewMode::List);
            }
            if ui.selectable_label(self.selected_view == TimelineViewMode::Calendar, "Calendar").clicked() {
                self.set_view_mode(TimelineViewMode::Calendar);
            }
        });
        
        // Filter selector
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui.selectable_label(self.filter == TimelineFilter::All, "All").clicked() {
                self.set_filter(TimelineFilter::All);
            }
            if ui.selectable_label(self.filter == TimelineFilter::InProgress, "In Progress").clicked() {
                self.set_filter(TimelineFilter::InProgress);
            }
            if ui.selectable_label(self.filter == TimelineFilter::Completed, "Completed").clicked() {
                self.set_filter(TimelineFilter::Completed);
            }
            if ui.selectable_label(self.filter == TimelineFilter::Unassigned, "Unassigned").clicked() {
                self.set_filter(TimelineFilter::Unassigned);
            }
            if ui.selectable_label(self.filter == TimelineFilter::Overdue, "Overdue").clicked() {
                self.set_filter(TimelineFilter::Overdue);
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Days to show:");
            ui.add(egui::Slider::new(&mut self.days_to_show, 7..=365));
            if ui.button("Reset").clicked() {
                self.days_to_show = 30;
            }
        });
        
        ui.separator();
        
        match self.selected_view {
            TimelineViewMode::Gantt => {
                // Gantt chart view
                ui.label("Gantt Chart View");
                
                // Convert arrays to HashMaps for the Gantt chart
                let task_map: HashMap<Uuid, Task> = tasks.iter()
                    .map(|t| (t.id, t.clone()))
                    .collect();
                
                // Create empty resources and schedule for now
                let resources = HashMap::new();
                let schedule = TimelineSchedule {
                    task_schedules: HashMap::new(),
                    resource_allocations: Vec::new(),
                    critical_path: Vec::new(),
                    warnings: Vec::new(),
                };
                
                // Render the Gantt chart
                self.gantt_chart.render(ui, &task_map, &resources, &schedule);
            },
            TimelineViewMode::List => {
                // List view
                let today = Utc::now();
                let end_date = today + Duration::days(self.days_to_show);
                
                ui.label("Scheduled Tasks:");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for task in tasks {
                        if self.should_show_task(task) {
                            if let Some(scheduled) = task.scheduled_date {
                                if scheduled >= today && scheduled <= end_date {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}: ", scheduled.format("%Y-%m-%d")));
                                        ui.label(&task.title);
                                        if task.is_overdue() {
                                            ui.colored_label(egui::Color32::RED, "(Overdue)");
                                        }
                                    });
                                }
                            }
                        }
                    }
                });
                
                ui.separator();
                
                ui.label("Goal Deadlines:");
                for goal in goals {
                    if let Some(target) = goal.target_date {
                        if target >= today && target <= end_date {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: ", target.format("%Y-%m-%d")));
                                ui.label(&goal.title);
                            });
                        }
                    }
                }
            },
            TimelineViewMode::Calendar => {
                // Calendar view
                ui.label("Calendar View (Coming Soon)");
            }
        }
        
        // Show warnings if any
        let task_map: HashMap<Uuid, Task> = tasks.iter()
            .map(|t| (t.id, t.clone()))
            .collect();
        let warnings = self.generate_warnings(&task_map);
        
        if !warnings.is_empty() {
            ui.separator();
            ui.collapsing("Warnings", |ui| {
                for warning in warnings {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "⚠");
                        ui.label(warning);
                    });
                }
            });
        }
    }
    
    fn should_show_task(&self, task: &Task) -> bool {
        match self.filter {
            TimelineFilter::All => true,
            TimelineFilter::InProgress => task.status == crate::domain::task::TaskStatus::InProgress,
            TimelineFilter::Completed => task.status == crate::domain::task::TaskStatus::Done,
            TimelineFilter::Unassigned => task.assigned_resource_id.is_none(),
            TimelineFilter::Overdue => task.is_overdue(),
        }
    }
}