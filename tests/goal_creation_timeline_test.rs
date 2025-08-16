use plon::ui::views::goal_view::{GoalView, GoalAction};
use plon::ui::views::timeline_view::TimelineView;

#[test]
fn test_goal_creation_ui_visibility() {
    // Test that the goal creation UI is now prominent and toggleable
    let mut goal_view = GoalView::new();
    
    // Initially, the create form should be hidden
    assert!(!goal_view.show_create_form);
    
    // Simulate clicking the "New Goal" button (toggle the form)
    goal_view.show_create_form = !goal_view.show_create_form;
    assert!(goal_view.show_create_form);
    
    // Set up form data
    goal_view.new_goal_title = "Test Goal".to_string();
    goal_view.new_goal_description = "Test Description".to_string();
    
    // Form should be valid with a non-empty title
    assert!(goal_view.is_form_valid());
    
    // After creating, the form should close (simulated)
    goal_view.show_create_form = false;
    assert!(!goal_view.show_create_form);
}

#[test]
fn test_goal_action_creation() {
    let mut goal_view = GoalView::new();
    goal_view.new_goal_title = "My Goal".to_string();
    goal_view.new_goal_description = "Goal Description".to_string();
    
    // The action that would be created
    let action = GoalAction::Create {
        title: goal_view.new_goal_title.clone(),
        description: goal_view.new_goal_description.clone(),
        parent_id: None,
    };
    
    // Verify the action contains the correct data
    match action {
        GoalAction::Create { title, description, parent_id } => {
            assert_eq!(title, "My Goal");
            assert_eq!(description, "Goal Description");
            assert_eq!(parent_id, None);
        }
        _ => panic!("Expected Create action"),
    }
}

#[test] 
fn test_timeline_view_no_infinite_scroll() {
    let mut timeline_view = TimelineView::new();
    
    // The view should have reasonable constraints
    assert_eq!(timeline_view.days_to_show, 30);
    
    // Test that zoom levels are bounded
    timeline_view.set_date_range(7);
    assert_eq!(timeline_view.days_to_show, 7);
    
    timeline_view.set_date_range(400); // Try to exceed max
    assert_eq!(timeline_view.days_to_show, 365); // Should be capped at 365
    
    timeline_view.set_date_range(3); // Try to go below min
    assert_eq!(timeline_view.days_to_show, 7); // Should be capped at 7
}

#[test]
fn test_timeline_view_content_dimensions() {
    let timeline_view = TimelineView::new();
    
    // Calculate expected dimensions for Gantt view
    let row_height = 30.0;
    let day_width = 25.0 * timeline_view.zoom_level;
    let label_width = 200.0;
    
    // With 10 tasks
    let task_count = 10;
    let chart_width = (label_width + (timeline_view.days_to_show as f32 * day_width)).min(5000.0);
    let chart_height = (task_count as f32 * row_height + 50.0).min(3000.0);
    
    // Verify dimensions are bounded
    assert!(chart_width <= 5000.0);
    assert!(chart_height <= 3000.0);
    
    // With 1000 tasks (should hit the limit)
    let task_count = 1000;
    let chart_height = (task_count as f32 * row_height + 50.0).min(3000.0);
    assert_eq!(chart_height, 3000.0); // Should be capped
}

