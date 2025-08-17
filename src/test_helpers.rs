// Test helpers for integration testing

use crate::ui::PlonApp;
use crate::repository::Repository;
use crate::services::{TaskService, GoalService, ResourceService, TaskConfigService};
use crate::domain::task::Task;
use std::sync::Arc;
use uuid::Uuid;

impl PlonApp {
    pub fn new_for_test() -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let repository = Arc::new(Repository::new_memory());
        
        let task_service = Arc::new(TaskService::new(repository.clone()));
        let goal_service = Arc::new(GoalService::new(repository.clone()));
        let resource_service = Arc::new(ResourceService::new(repository.clone()));
        let task_config_service = Arc::new(TaskConfigService::new(repository.clone()));
        
        Self {
            repository: repository.clone(),
            task_service,
            goal_service,
            resource_service,
            task_config_service,
            
            current_view: crate::ui::ViewType::List,
            selected_task_id: None,
            selected_goal_id: None,
            show_task_editor: false,
            show_goal_editor: false,
            
            list_view: crate::ui::views::list_view::ListView::new(),
            kanban_view: crate::ui::views::kanban_view_enhanced::KanbanView::new(),
            map_view: crate::ui::views::map_view::MapView::new(),
            timeline_view: crate::ui::views::timeline_view::TimelineView::new(),
            dashboard_view: crate::ui::views::dashboard_view::DashboardView::new(),
            recurring_view: crate::ui::views::recurring_view::RecurringView::new(),
            metadata_config_view: crate::ui::views::metadata_config_view::MetadataConfigView::new(),
            resource_view: crate::ui::views::resource_view::ResourceView::new(),
            gantt_view: crate::ui::views::gantt_view::GanttView::new(),
            goal_view: crate::ui::views::goal_view::GoalView::new(),
            
            tasks: Vec::new(),
            goals: Vec::new(),
            resources: Vec::new(),
            dependencies: Vec::new(),
            
            runtime,
        }
    }
    
    pub fn has_improved_kanban_view(&self) -> bool {
        true // Type system ensures we're using kanban_view_enhanced::KanbanView
    }
    
    pub fn add_test_task(&mut self, task: Task) {
        self.tasks.push(task.clone());
        self.kanban_view.add_task(task);
    }
    
    pub fn switch_to_kanban_view(&mut self) {
        self.current_view = crate::ui::ViewType::Kanban;
    }
    
    pub fn is_kanban_view_active(&self) -> bool {
        self.current_view == crate::ui::ViewType::Kanban
    }
    
    pub fn can_start_drag_in_kanban(&self, task_id: Uuid) -> bool {
        self.tasks.iter().any(|t| t.id == task_id)
    }
    
    pub fn start_kanban_drag(&mut self, task_id: Uuid, pos: eframe::egui::Pos2) {
        self.kanban_view.start_drag(task_id, pos);
    }
    
    pub fn is_kanban_dragging(&self) -> bool {
        self.kanban_view.is_dragging()
    }
    
    pub fn update_kanban_drag(&mut self, pos: eframe::egui::Pos2) {
        self.kanban_view.update_drag_position(pos);
    }
    
    pub fn complete_kanban_drag(&mut self, column: usize) {
        self.kanban_view.complete_drag(column);
        // Sync the task status back
        for task in &self.kanban_view.tasks {
            if let Some(app_task) = self.tasks.iter_mut().find(|t| t.id == task.id) {
                app_task.status = task.status;
            }
        }
    }
    
    pub fn get_task(&self, id: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    
    pub fn get_kanban_column_count(&self) -> usize {
        self.kanban_view.columns.len()
    }
    
    pub fn get_kanban_tasks_in_column(&self, column_index: usize) -> Vec<&Task> {
        self.kanban_view.get_tasks_for_column(column_index)
    }
    
    pub fn enable_kanban_quick_add(&mut self, column_index: usize) {
        self.kanban_view.enable_quick_add(column_index);
    }
    
    pub fn is_kanban_quick_add_active(&self, column_index: usize) -> bool {
        self.kanban_view.is_quick_add_active(column_index)
    }
    
    pub fn kanban_quick_add_task(&mut self, column_index: usize, title: String) {
        self.kanban_view.quick_add_task(column_index, title.clone());
        // Sync with app tasks
        if let Some(new_task) = self.kanban_view.tasks.last() {
            if !self.tasks.iter().any(|t| t.id == new_task.id) {
                self.tasks.push(new_task.clone());
            }
        }
    }
    
    pub fn select_kanban_task(&mut self, task_id: Uuid) {
        self.kanban_view.select_task(task_id);
    }
    
    pub fn get_selected_kanban_task(&self) -> Option<Uuid> {
        self.kanban_view.get_selected_task_id()
    }
    
    pub fn clear_kanban_selection(&mut self) {
        self.kanban_view.clear_selection();
    }
    
    // Direct access to kanban view for comprehensive testing
    pub fn get_kanban_view(&self) -> &crate::ui::views::kanban_view_enhanced::KanbanView {
        &self.kanban_view
    }
    
    pub fn get_kanban_view_mut(&mut self) -> &mut crate::ui::views::kanban_view_enhanced::KanbanView {
        &mut self.kanban_view
    }
    
    // Test sync between app tasks and kanban view
    pub fn sync_tasks_to_kanban(&mut self) {
        // Clear kanban tasks and re-add from app tasks
        self.kanban_view.tasks.clear();
        for column in &mut self.kanban_view.columns {
            column.task_order.clear();
        }
        
        for task in &self.tasks {
            self.kanban_view.add_task(task.clone());
        }
    }
    
    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
    
    pub fn get_tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }
}