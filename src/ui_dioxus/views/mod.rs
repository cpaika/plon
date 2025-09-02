// Simple versions (kept as fallback)
pub mod map_view_simple;
pub mod list_view_simple;
// pub mod list_view_optimized; // Temporarily disabled due to compilation issues
pub mod kanban_view_simple;
pub mod kanban_view_fixed;
pub mod kanban_view_with_db;
pub mod kanban_view_ordered;
pub mod timeline_view_simple;
pub mod gantt_view_simple;
// pub mod gantt_view_enhanced; // Temporarily disabled due to lifetime issues
pub mod dashboard;
pub mod settings_view;
pub mod goals_view;

#[cfg(test)]
mod list_view_test;
#[cfg(test)]
mod list_view_edit_test;

// Working versions with actual functionality  
pub mod map_working;
pub mod map_with_dependencies;
pub mod map_with_drag_dependencies;
pub mod map_with_animated_dependencies;
pub mod map_final;

// Use working versions where available
pub use map_final::MapView;
pub use list_view_simple::ListView;
pub use kanban_view_ordered::KanbanViewOrdered as KanbanView;  // Use the ordered version
pub use timeline_view_simple::TimelineView;
pub use gantt_view_simple::GanttView;
pub use dashboard::Dashboard;
pub use settings_view::SettingsView;
pub use goals_view::GoalsView;