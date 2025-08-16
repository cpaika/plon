use plon::domain::task::{Task, TaskStatus, Priority};
use plon::ui::views::kanban_view::{KanbanView, FilterOptions};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::TaskService;
use std::sync::Arc;
use chrono::Utc;

#[tokio::test]
async fn test_enhanced_kanban_features() {
    // Initialize database and services
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create test tasks with various properties
    let mut critical_task = Task::new("Critical Security Fix".to_string(), "Fix authentication bypass".to_string());
    critical_task.priority = Priority::Critical;
    critical_task.status = TaskStatus::InProgress;
    critical_task.add_tag("security".to_string());
    critical_task.add_tag("urgent".to_string());
    critical_task.due_date = Some(Utc::now() + chrono::Duration::days(1));
    
    let mut normal_task = Task::new("Add User Profile".to_string(), "Implement user profile page".to_string());
    normal_task.priority = Priority::Medium;
    normal_task.status = TaskStatus::Todo;
    normal_task.add_tag("feature".to_string());
    normal_task.add_subtask("Design UI".to_string());
    normal_task.add_subtask("Implement backend".to_string());
    normal_task.add_subtask("Write tests".to_string());
    
    let mut blocked_task = Task::new("Integration Task".to_string(), "Waiting for API".to_string());
    blocked_task.status = TaskStatus::Blocked;
    blocked_task.add_tag("blocked".to_string());
    
    let mut overdue_task = Task::new("Documentation Update".to_string(), "Update API docs".to_string());
    overdue_task.priority = Priority::High;
    overdue_task.due_date = Some(Utc::now() - chrono::Duration::days(2));
    overdue_task.status = TaskStatus::Review;
    
    // Save tasks
    let critical_saved = service.create(critical_task.clone()).await.unwrap();
    let normal_saved = service.create(normal_task.clone()).await.unwrap();
    let blocked_saved = service.create(blocked_task.clone()).await.unwrap();
    let overdue_saved = service.create(overdue_task.clone()).await.unwrap();
    
    // Test Kanban view initialization
    let mut view = KanbanView::new();
    
    // Test filtering
    let mut all_tasks = vec![critical_saved.clone(), normal_saved.clone(), blocked_saved.clone(), overdue_saved.clone()];
    
    // Filter by priority
    let mut priority_filter = FilterOptions::default();
    priority_filter.priorities = vec![Priority::Critical, Priority::High];
    let filtered = view.apply_filters(&all_tasks, &priority_filter);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().any(|t| t.priority == Priority::Critical));
    assert!(filtered.iter().any(|t| t.priority == Priority::High));
    
    // Filter by tags
    let mut tag_filter = FilterOptions::default();
    tag_filter.tags = vec!["security".to_string()];
    let filtered = view.apply_filters(&all_tasks, &tag_filter);
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].tags.contains(&"security".to_string()));
    
    // Filter by search text
    let mut search_filter = FilterOptions::default();
    search_filter.search_text = Some("documentation".to_string());
    let filtered = view.apply_filters(&all_tasks, &search_filter);
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].title.to_lowercase().contains("documentation"));
    
    // Test swimlane organization
    let swimlanes = view.organize_into_swimlanes(&all_tasks);
    assert!(!swimlanes.is_empty());
    
    // Test WIP limits
    view.set_wip_limit("In Progress", 3);
    assert_eq!(view.get_wip_limit("In Progress"), Some(3));
    
    // Enable swimlanes
    view.enable_swimlanes_by_priority();
    let priority_swimlanes = view.organize_into_swimlanes(&all_tasks);
    assert!(priority_swimlanes.contains_key("Critical"));
    assert!(priority_swimlanes.contains_key("High"));
    assert!(priority_swimlanes.contains_key("Medium"));
    
    // Test card style for critical task
    let style = view.get_card_style(&critical_saved);
    assert_eq!(style.border_width, 3.0);
    assert!(style.show_overdue_badge == false);
    
    // Test card style for overdue task
    let overdue_style = view.get_card_style(&overdue_saved);
    assert!(overdue_style.show_overdue_badge);
    assert!(overdue_style.pulse_animation);
    
    // Test card style for blocked task
    let blocked_style = view.get_card_style(&blocked_saved);
    assert!(blocked_style.show_blocked_overlay);
    assert_eq!(blocked_style.opacity, 0.8);
    
    // Verify subtask progress
    let (completed, total) = normal_saved.subtask_progress();
    assert_eq!(total, 3);
    assert_eq!(completed, 0);
    
    println!("✅ All enhanced Kanban features tested successfully!");
}

#[tokio::test]
async fn test_kanban_column_management() {
    let mut view = KanbanView::new();
    
    // Test column visibility
    view.set_column_visible("Review", false);
    assert!(!view.is_column_visible("Review"));
    
    view.set_column_visible("Review", true);
    assert!(view.is_column_visible("Review"));
    
    // Test column width
    view.set_column_width("Todo", 300.0);
    assert_eq!(view.get_column_width("Todo"), 300.0);
    
    // Test column collapse/expand
    view.collapse_column("In Progress");
    assert!(view.is_column_collapsed("In Progress"));
    assert_eq!(view.get_column_width("In Progress"), 50.0);
    
    view.expand_column("In Progress");
    assert!(!view.is_column_collapsed("In Progress"));
    
    // Test adding custom columns
    view.add_custom_column("Backlog", TaskStatus::Todo, (128, 128, 128, 255));
    view.add_custom_column("QA", TaskStatus::Review, (255, 200, 100, 255));
    
    let columns = view.get_columns();
    assert!(columns.iter().any(|c| c.title == "Backlog"));
    assert!(columns.iter().any(|c| c.title == "QA"));
    
    println!("✅ Column management features tested successfully!");
}

#[tokio::test]
async fn test_task_filtering_and_search() {
    let view = KanbanView::new();
    
    // Create test tasks
    let mut tasks = Vec::new();
    
    let mut frontend_task = Task::new("Frontend Feature".to_string(), "React component".to_string());
    frontend_task.add_tag("frontend".to_string());
    frontend_task.priority = Priority::High;
    tasks.push(frontend_task);
    
    let mut backend_task = Task::new("Backend API".to_string(), "REST endpoint".to_string());
    backend_task.add_tag("backend".to_string());
    backend_task.priority = Priority::Medium;
    tasks.push(backend_task);
    
    let mut bug_task = Task::new("Fix Bug".to_string(), "Critical bug in auth".to_string());
    bug_task.add_tag("bug".to_string());
    bug_task.priority = Priority::Critical;
    tasks.push(bug_task);
    
    // Test text search
    let results = view.filter_tasks(&tasks, "bug");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.to_lowercase().contains("bug"));
    
    // Test tag filtering
    let mut filter = FilterOptions::default();
    filter.tags = vec!["frontend".to_string()];
    let filtered = view.apply_filters(&tasks, &filter);
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].tags.contains(&"frontend".to_string()));
    
    // Test priority filtering
    filter.tags.clear();
    filter.priorities = vec![Priority::Critical, Priority::High];
    let filtered = view.apply_filters(&tasks, &filter);
    assert_eq!(filtered.len(), 2);
    
    // Test combined filters
    filter.tags = vec!["bug".to_string()];
    filter.priorities = vec![Priority::Critical];
    let filtered = view.apply_filters(&tasks, &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Fix Bug");
    
    println!("✅ Filtering and search features tested successfully!");
}

#[tokio::test]
async fn test_visual_enhancements() {
    let mut view = KanbanView::new();
    
    // Test priority colors
    let mut task = Task::new("Test".to_string(), "".to_string());
    
    task.priority = Priority::Critical;
    let style = view.get_card_style(&task);
    assert_eq!(style.priority_indicator_color.r(), 255);
    assert_eq!(style.border_width, 3.0);
    
    task.priority = Priority::High;
    let style = view.get_card_style(&task);
    assert_eq!(style.priority_indicator_color.r(), 255);
    assert_eq!(style.priority_indicator_color.g(), 149);
    
    task.priority = Priority::Medium;
    let style = view.get_card_style(&task);
    assert_eq!(style.priority_indicator_color.r(), 52);
    assert_eq!(style.priority_indicator_color.g(), 199);
    
    task.priority = Priority::Low;
    let style = view.get_card_style(&task);
    assert_eq!(style.priority_indicator_color.r(), 175);
    
    // Test overdue styling
    task.due_date = Some(Utc::now() - chrono::Duration::days(1));
    task.status = TaskStatus::InProgress;
    let style = view.get_card_style(&task);
    assert!(style.show_overdue_badge);
    assert!(style.pulse_animation);
    
    // Test blocked task styling
    task.status = TaskStatus::Blocked;
    let style = view.get_card_style(&task);
    assert!(style.show_blocked_overlay);
    assert_eq!(style.blocked_pattern, "diagonal_stripes");
    assert_eq!(style.opacity, 0.8);
    
    // Test tag color assignment
    let tags = vec!["frontend", "backend", "bug", "feature", "documentation"];
    let colors = view.assign_tag_colors(&tags);
    assert_eq!(colors.len(), 5);
    for tag in &tags {
        assert!(colors.contains_key(&tag.to_string()));
    }
    
    println!("✅ Visual enhancement features tested successfully!");
}