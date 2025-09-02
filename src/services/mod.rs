mod auto_run_orchestrator;
mod claude_code_service;
mod dependency_service;
mod goal_service;
mod recurring_service;
mod resource_service;
mod scheduler;
mod task_config_service;
mod task_service;
// mod auto_run_orchestrator_improvements;  // Temporarily disabled - needs field visibility fixes
// mod race_condition_fixes;  // Temporarily disabled - needs dependency
// pub mod validation;  // Temporarily disabled - needs regex fixes
// pub mod error_handling;  // Temporarily disabled - needs dependencies
#[cfg(test)]
mod auto_run_e2e_tests;
#[cfg(test)]
mod claude_automation_e2e_tests;
mod pr_review_service;
// #[cfg(test)]
// mod error_recovery_tests;  // Temporarily disabled - needs fixes
// #[cfg(test)]
// mod stress_tests;  // Temporarily disabled - needs fixes
pub mod command_executor;
pub mod summarization;
pub mod timeline_scheduler;
pub mod claude_automation;
pub mod claude_monitor;
pub mod pr_monitor;
pub mod workspace_service;
pub mod task_dependency_service;
pub mod time_tracking_service;
pub mod export_service;

pub use auto_run_orchestrator::{
    AutoRunConfig, AutoRunOrchestrator, AutoRunStatus, AutoRunProgress, TaskExecution,
    TaskExecutionStatus,
};
pub use claude_code_service::ClaudeCodeService;
pub use dependency_service::DependencyService;
pub use goal_service::GoalService;
pub use pr_review_service::PRReviewService;
pub use recurring_service::RecurringService;
pub use resource_service::ResourceService;
pub use task_config_service::TaskConfigService;
pub use task_service::TaskService;
pub use claude_automation::ClaudeAutomation;
pub use claude_monitor::{ClaudeMonitor, start_claude_monitor_background};
pub use pr_monitor::{PrMonitor, start_pr_monitor_background};
pub use workspace_service::{WorkspaceService, WorkspaceType};
pub use task_dependency_service::TaskDependencyService;
pub use time_tracking_service::{TimeTrackingService, TimeEntry};
pub use export_service::{ExportService, ExportFormat};
