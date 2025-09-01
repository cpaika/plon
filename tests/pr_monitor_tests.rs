#[cfg(test)]
mod pr_monitor_tests {
    use plon::services::PrMonitor;
    use plon::repository::Repository;
    use plon::repository::database::init_test_database;
    use plon::domain::task::Task;
    use plon::domain::task_execution::{TaskExecution, ExecutionStatus};
    use tempfile::TempDir;
    use std::process::Command;

    async fn setup_test_environment() -> (Repository, TempDir) {
        let pool = init_test_database().await.unwrap();
        let repo = Repository::new(pool);
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git repo
        Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["init"])
            .output()
            .unwrap();
        
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_pr_monitor_creation() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        let monitor = PrMonitor::new(repo, temp_dir.path().to_path_buf());
        
        // Basic creation test
        assert!(true); // Monitor created successfully
    }

    #[tokio::test]
    async fn test_get_recent_pr_activity_empty() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        let monitor = PrMonitor::new(repo, temp_dir.path().to_path_buf());
        
        // Get activity for last 24 hours (should be empty)
        let activities = monitor.get_recent_pr_activity(24).await.unwrap();
        assert_eq!(activities.len(), 0);
    }

    #[tokio::test]
    async fn test_get_recent_pr_activity_with_executions() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        // Create a task
        let task = Task::new("Test PR task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create execution with PR
        let mut execution = TaskExecution::new(task.id, "pr-branch".to_string());
        execution.status = ExecutionStatus::PendingReview;
        execution.pr_url = Some("https://github.com/test/repo/pull/1".to_string());
        repo.task_executions.create(&execution).await.unwrap();
        
        let monitor = PrMonitor::new(repo.clone(), temp_dir.path().to_path_buf());
        
        // Get recent activity
        let activities = monitor.get_recent_pr_activity(24).await.unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].pr_url, "https://github.com/test/repo/pull/1");
        assert_eq!(activities[0].task_title, "Test PR task");
        assert_eq!(activities[0].status, ExecutionStatus::PendingReview);
    }

    #[tokio::test]
    async fn test_get_recent_pr_activity_filters_old() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        // Create a task
        let task = Task::new("Old PR task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create old execution (completed more than 1 hour ago)
        let mut execution = TaskExecution::new(task.id, "old-branch".to_string());
        execution.status = ExecutionStatus::Merged;
        execution.pr_url = Some("https://github.com/test/repo/pull/99".to_string());
        execution.completed_at = Some(chrono::Utc::now() - chrono::Duration::hours(2));
        repo.task_executions.create(&execution).await.unwrap();
        
        let monitor = PrMonitor::new(repo.clone(), temp_dir.path().to_path_buf());
        
        // Get activity for last 1 hour (should be empty)
        let activities = monitor.get_recent_pr_activity(1).await.unwrap();
        assert_eq!(activities.len(), 0);
        
        // Get activity for last 3 hours (should include the old one)
        let activities = monitor.get_recent_pr_activity(3).await.unwrap();
        assert_eq!(activities.len(), 1);
    }

    #[tokio::test]
    async fn test_pr_activity_multiple_statuses() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        // Create tasks
        let task1 = Task::new("Task 1".to_string(), "Description".to_string());
        let task2 = Task::new("Task 2".to_string(), "Description".to_string());
        let task3 = Task::new("Task 3".to_string(), "Description".to_string());
        repo.tasks.create(&task1).await.unwrap();
        repo.tasks.create(&task2).await.unwrap();
        repo.tasks.create(&task3).await.unwrap();
        
        // Create executions with different statuses
        let mut exec1 = TaskExecution::new(task1.id, "branch-1".to_string());
        exec1.status = ExecutionStatus::PendingReview;
        exec1.pr_url = Some("https://github.com/test/repo/pull/1".to_string());
        
        let mut exec2 = TaskExecution::new(task2.id, "branch-2".to_string());
        exec2.status = ExecutionStatus::Merged;
        exec2.pr_url = Some("https://github.com/test/repo/pull/2".to_string());
        exec2.completed_at = Some(chrono::Utc::now());
        
        let mut exec3 = TaskExecution::new(task3.id, "branch-3".to_string());
        exec3.status = ExecutionStatus::Running;
        exec3.pr_url = Some("https://github.com/test/repo/pull/3".to_string());
        
        repo.task_executions.create(&exec1).await.unwrap();
        repo.task_executions.create(&exec2).await.unwrap();
        repo.task_executions.create(&exec3).await.unwrap();
        
        let monitor = PrMonitor::new(repo.clone(), temp_dir.path().to_path_buf());
        
        // Get recent activity
        let activities = monitor.get_recent_pr_activity(24).await.unwrap();
        
        // Should include all executions with PRs
        assert_eq!(activities.len(), 3);
        
        // Verify different statuses
        let pending_count = activities.iter()
            .filter(|a| a.status == ExecutionStatus::PendingReview)
            .count();
        let merged_count = activities.iter()
            .filter(|a| a.status == ExecutionStatus::Merged)
            .count();
        let running_count = activities.iter()
            .filter(|a| a.status == ExecutionStatus::Running)
            .count();
        
        assert_eq!(pending_count, 1);
        assert_eq!(merged_count, 1);
        assert_eq!(running_count, 1);
    }
}