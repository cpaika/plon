use chrono::{Duration, Utc};
use plon::domain::goal::{Goal, GoalStatus};
use plon::repository::{Repository, goal_repository::GoalRepository};
use plon::services::GoalService;
use plon::ui::views::goal_view::GoalView;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod goal_creation_tests {
    use super::*;

    #[test]
    fn test_goal_creation_basic() {
        let goal = Goal::new(
            "Launch Product".to_string(),
            "Successfully launch the new product line".to_string(),
        );

        assert!(!goal.title.is_empty());
        assert!(!goal.description.is_empty());
        assert_eq!(goal.status, GoalStatus::NotStarted);
        assert!(goal.id != Uuid::nil());
        assert!(goal.target_date.is_none());
        assert!(goal.parent_goal_id.is_none());
        assert!(goal.task_ids.is_empty());
    }

    #[test]
    fn test_goal_creation_with_target_date() {
        let mut goal = Goal::new(
            "Q1 Revenue Target".to_string(),
            "Achieve $1M in revenue".to_string(),
        );

        let target = Utc::now() + Duration::days(90);
        goal.target_date = Some(target);

        assert_eq!(goal.target_date, Some(target));
        assert!(goal.days_until_target().is_some());
    }

    #[test]
    fn test_goal_creation_with_parent() {
        let parent = Goal::new("Annual Goals".to_string(), "Goals for the year".to_string());

        let mut child = Goal::new("Q1 Goals".to_string(), "First quarter goals".to_string());

        child.parent_goal_id = Some(parent.id);

        assert_eq!(child.parent_goal_id, Some(parent.id));
    }

    #[test]
    fn test_goal_creation_with_tasks() {
        let mut goal = Goal::new(
            "Technical Debt Reduction".to_string(),
            "Reduce technical debt by 50%".to_string(),
        );

        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();

        goal.add_task(task1);
        goal.add_task(task2);
        goal.add_task(task3);

        assert_eq!(goal.task_ids.len(), 3);
        assert!(goal.task_ids.contains(&task1));
        assert!(goal.task_ids.contains(&task2));
    }

    #[test]
    fn test_goal_status_transitions() {
        let mut goal = Goal::new(
            "Implement Feature X".to_string(),
            "Complete implementation of feature X".to_string(),
        );

        // Initial status
        assert_eq!(goal.status, GoalStatus::NotStarted);

        // Start working on goal
        goal.status = GoalStatus::Active;
        assert_eq!(goal.status, GoalStatus::Active);

        // Complete the goal
        goal.status = GoalStatus::Completed;
        assert_eq!(goal.status, GoalStatus::Completed);
    }

    #[test]
    fn test_goal_hierarchy() {
        let mut goals = Vec::new();

        // Create parent goal
        let parent = Goal::new(
            "Company OKRs".to_string(),
            "Objectives and Key Results".to_string(),
        );
        let parent_id = parent.id;
        goals.push(parent);

        // Create child goals
        for i in 1..=3 {
            let mut child = Goal::new(format!("Objective {}", i), format!("Key result {}", i));
            child.parent_goal_id = Some(parent_id);
            goals.push(child);
        }

        // Verify hierarchy
        let children: Vec<&Goal> = goals
            .iter()
            .filter(|g| g.parent_goal_id == Some(parent_id))
            .collect();

        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_goal_progress_calculation() {
        let mut goal = Goal::new(
            "Complete Sprint".to_string(),
            "Finish all sprint tasks".to_string(),
        );

        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();

        goal.add_task(task1);
        goal.add_task(task2);
        goal.add_task(task3);

        // No tasks completed
        let progress = goal.calculate_progress(&vec![]);
        assert_eq!(progress, 0.0);

        // One task completed
        let progress = goal.calculate_progress(&vec![(task1, true)]);
        assert!((progress - 33.33).abs() < 0.1);

        // All tasks completed
        let progress = goal.calculate_progress(&vec![(task1, true), (task2, true), (task3, true)]);
        assert_eq!(progress, 100.0);
    }

    #[test]
    fn test_goal_validation() {
        // Test empty title
        let goal = Goal::new("".to_string(), "Description".to_string());
        assert!(
            goal.title.is_empty(),
            "Should allow empty title but mark as invalid"
        );

        // Test very long title
        let long_title = "a".repeat(1000);
        let goal = Goal::new(long_title.clone(), "Description".to_string());
        assert_eq!(goal.title, long_title);
    }

    #[tokio::test]
    async fn test_goal_repository_create() {
        use sqlx::SqlitePool;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = GoalRepository::new(Arc::new(pool.clone()));

        let goal = Goal::new("Test Goal".to_string(), "Test Description".to_string());

        let result = repo.create(&goal).await;
        assert!(result.is_ok(), "Failed to create goal: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_goal_repository_update() {
        use sqlx::SqlitePool;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = GoalRepository::new(Arc::new(pool.clone()));

        let mut goal = Goal::new(
            "Original Title".to_string(),
            "Original Description".to_string(),
        );

        let result = repo.create(&goal).await;
        assert!(result.is_ok(), "Failed to create goal: {:?}", result.err());

        // Update the goal
        let mut updated = goal.clone();
        updated.title = "Updated Title".to_string();
        updated.status = GoalStatus::Active;

        let result = repo.update(&updated).await;
        assert!(result.is_ok(), "Failed to update goal: {:?}", result.err());

        // Verify update worked by fetching
        let goals = repo.list_all().await.unwrap();
        let fetched = goals.iter().find(|g| g.id == goal.id).unwrap();
        assert_eq!(fetched.title, "Updated Title");
        assert_eq!(fetched.status, GoalStatus::Active);
    }

    #[tokio::test]
    async fn test_goal_service_create_with_validation() {
        use sqlx::SqlitePool;
        use std::sync::Arc;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = Arc::new(Repository::new(pool.clone()));
        let service = GoalService::new(repo.clone());

        // Test creating goal with service
        let goal = Goal::new(
            "Service Goal".to_string(),
            "Created via service".to_string(),
        );
        let result = service.create(goal).await;

        assert!(
            result.is_ok(),
            "Failed to create goal via service: {:?}",
            result.err()
        );

        let goal = result.unwrap();
        assert_eq!(goal.title, "Service Goal");
        assert_eq!(goal.status, GoalStatus::NotStarted);
    }

    #[test]
    fn test_goal_view_initialization() {
        let view = GoalView::new();

        assert!(view.new_goal_title.is_empty());
        assert!(view.new_goal_description.is_empty());
        assert!(view.selected_goal_id.is_none());
        assert!(view.show_archived == false);
    }

    #[test]
    fn test_goal_view_form_validation() {
        let mut view = GoalView::new();

        // Empty title should be invalid
        view.new_goal_title = "".to_string();
        view.new_goal_description = "Description".to_string();
        assert!(!view.is_form_valid(), "Empty title should be invalid");

        // Valid form
        view.new_goal_title = "Valid Title".to_string();
        assert!(view.is_form_valid(), "Non-empty title should be valid");

        // Very long title should still be valid (truncation happens on save)
        view.new_goal_title = "a".repeat(500);
        assert!(view.is_form_valid(), "Long title should be valid");
    }

    #[test]
    fn test_goal_view_clear_form() {
        let mut view = GoalView::new();

        view.new_goal_title = "Test".to_string();
        view.new_goal_description = "Desc".to_string();
        view.selected_parent_id = Some(Uuid::new_v4());

        view.clear_form();

        assert!(view.new_goal_title.is_empty());
        assert!(view.new_goal_description.is_empty());
        assert!(view.selected_parent_id.is_none());
    }
}
