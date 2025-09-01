use fermi::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::domain::goal::Goal;
use crate::domain::dependency::Dependency;
use crate::repository::Repository;
use crate::services::{TaskService, DependencyService, ClaudeCodeService};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

// Global state atoms using Fermi for state management
pub static TASKS: Atom<Vec<Task>> = Atom(|_| Vec::new());
pub static GOALS: Atom<Vec<Goal>> = Atom(|_| Vec::new());
pub static DEPENDENCIES: Atom<Vec<Dependency>> = Atom(|_| Vec::new());
pub static SELECTED_TASK: Atom<Option<Uuid>> = Atom(|_| None);
pub static SELECTED_GOAL: Atom<Option<Uuid>> = Atom(|_| None);
pub static CURRENT_VIEW: Atom<ViewType> = Atom(|_| ViewType::Map);
pub static ZOOM_LEVEL: Atom<f32> = Atom(|_| 1.0);
pub static CAMERA_POSITION: Atom<(f32, f32)> = Atom(|_| (0.0, 0.0));
pub static RUNNING_TASKS: Atom<HashMap<Uuid, TaskExecutionStatus>> = Atom(|_| HashMap::new());

#[derive(Clone, Debug, PartialEq)]
pub enum ViewType {
    Map,
    List,
    Kanban,
    Timeline,
    Gantt,
    Dashboard,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskExecutionStatus {
    Running,
    Completed,
    Failed(String),
}

// Application state manager
pub struct AppState {
    pub repository: Arc<Repository>,
    pub task_service: Arc<TaskService>,
    pub dependency_service: Arc<DependencyService>,
    pub claude_service: Arc<ClaudeCodeService>,
}

impl AppState {
    pub fn new() -> Self {
        // This would be properly initialized with real services
        // For now, we'll panic as this needs to be set up properly
        panic!("AppState needs to be initialized with proper services")
    }
    
    pub fn with_services(
        repository: Arc<Repository>,
        task_service: Arc<TaskService>,
        dependency_service: Arc<DependencyService>,
        claude_service: Arc<ClaudeCodeService>,
    ) -> Self {
        Self {
            repository,
            task_service,
            dependency_service,
            claude_service,
        }
    }
    
    pub async fn load_tasks(&self) -> Vec<Task> {
        self.task_service.list(Default::default())
            .await
            .unwrap_or_default()
    }
    
    pub async fn load_goals(&self) -> Vec<Goal> {
        self.repository.goals.list()
            .await
            .unwrap_or_default()
    }
    
    pub async fn load_dependencies(&self) -> Vec<Dependency> {
        self.dependency_service.list_all()
            .await
            .unwrap_or_default()
    }
    
    pub async fn create_task(&self, task: Task) -> Result<Task, String> {
        self.task_service.create(&task)
            .await
            .map_err(|e| e.to_string())
    }
    
    pub async fn update_task(&self, task: &Task) -> Result<(), String> {
        self.task_service.update(task)
            .await
            .map_err(|e| e.to_string())
    }
    
    pub async fn delete_task(&self, task_id: Uuid) -> Result<(), String> {
        self.task_service.delete(task_id)
            .await
            .map_err(|e| e.to_string())
    }
    
    pub async fn start_claude_code(&self, task_id: Uuid) -> Result<(), String> {
        // This would start Claude Code for the task
        Ok(())
    }
}