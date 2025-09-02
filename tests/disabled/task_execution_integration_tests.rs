#[cfg(test)]
mod task_execution_integration_tests {
    use plon::repository::Repository;
    use plon::repository::database::init_test_database;
    use plon::domain::task::{Task, TaskStatus, Priority};
    use plon::domain::task_execution::{TaskExecution, ExecutionStatus};
    use plon::services::ClaudeAutomation;
    use uuid::Uuid;
    use std::path::PathBuf;
    use chrono::Utc;

    async fn setup_test_repo() -> Repository {
        let pool = init_test_database().await.unwrap();
        Repository::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_retrieve_task_execution() {
        let repo = setup_test_repo().await;
        
        // Create a task first
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create a task execution
        let execution = TaskExecution::new(task.id, "test-branch".to_string());
        repo.task_executions.create(&execution).await.unwrap();
        
        // Retrieve the execution
        let retrieved = repo.task_executions.get(execution.id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, execution.id);
        assert_eq!(retrieved.task_id, task.id);
        assert_eq!(retrieved.branch_name, "test-branch");
        assert_eq!(retrieved.status, ExecutionStatus::Running);
    }

    #[tokio::test]
    async fn test_update_task_execution() {
        let repo = setup_test_repo().await;
        
        // Create a task and execution
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        let mut execution = TaskExecution::new(task.id, "test-branch".to_string());
        repo.task_executions.create(&execution).await.unwrap();
        
        // Update execution to success
        execution.complete_success(Some("https://github.com/test/pr/1".to_string()));
        repo.task_executions.update(&execution).await.unwrap();
        
        // Verify update
        let retrieved = repo.task_executions.get(execution.id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Success);
        assert_eq!(retrieved.pr_url, Some("https://github.com/test/pr/1".to_string()));
        assert!(retrieved.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_list_executions_for_task() {
        let repo = setup_test_repo().await;
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create multiple executions
        let exec1 = TaskExecution::new(task.id, "branch-1".to_string());
        let exec2 = TaskExecution::new(task.id, "branch-2".to_string());
        let exec3 = TaskExecution::new(task.id, "branch-3".to_string());
        
        repo.task_executions.create(&exec1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        repo.task_executions.create(&exec2).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        repo.task_executions.create(&exec3).await.unwrap();
        
        // List executions for the task
        let executions = repo.task_executions.list_for_task(task.id).await.unwrap();
        assert_eq!(executions.len(), 3);
        
        // Should be ordered by started_at DESC (most recent first)
        assert_eq!(executions[0].branch_name, "branch-3");
        assert_eq!(executions[1].branch_name, "branch-2");
        assert_eq!(executions[2].branch_name, "branch-1");
    }

    #[tokio::test]
    async fn test_get_active_execution_for_task() {
        let repo = setup_test_repo().await;
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create executions with different statuses
        let mut exec1 = TaskExecution::new(task.id, "branch-1".to_string());
        exec1.complete_success(None);
        repo.task_executions.create(&exec1).await.unwrap();
        
        let mut exec2 = TaskExecution::new(task.id, "branch-2".to_string());
        exec2.status = ExecutionStatus::PendingReview;
        repo.task_executions.create(&exec2).await.unwrap();
        
        // Get active execution (should return the PendingReview one)
        let active = repo.task_executions.get_active_for_task(task.id).await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().branch_name, "branch-2");
    }

    #[tokio::test]
    async fn test_list_recent_executions() {
        let repo = setup_test_repo().await;
        
        // Create multiple tasks and executions
        for i in 0..5 {
            let task = Task::new(format!("Task {}", i), "Description".to_string());
            repo.tasks.create(&task).await.unwrap();
            
            let execution = TaskExecution::new(task.id, format!("branch-{}", i));
            repo.task_executions.create(&execution).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // List recent with limit
        let recent = repo.task_executions.list_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
        
        // Should be ordered by started_at DESC
        assert_eq!(recent[0].branch_name, "branch-4");
        assert_eq!(recent[1].branch_name, "branch-3");
        assert_eq!(recent[2].branch_name, "branch-2");
    }

    #[tokio::test]
    async fn test_execution_with_logs() {
        let repo = setup_test_repo().await;
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create execution with logs
        let mut execution = TaskExecution::new(task.id, "test-branch".to_string());
        execution.add_log("Starting execution".to_string());
        execution.add_log("Running tests".to_string());
        execution.add_log("Tests passed".to_string());
        
        repo.task_executions.create(&execution).await.unwrap();
        
        // Retrieve and verify logs
        let retrieved = repo.task_executions.get(execution.id).await.unwrap().unwrap();
        assert_eq!(retrieved.output_log.len(), 3);
        assert!(retrieved.output_log[0].contains("Starting execution"));
        assert!(retrieved.output_log[1].contains("Running tests"));
        assert!(retrieved.output_log[2].contains("Tests passed"));
    }

    #[tokio::test]
    async fn test_execution_failure_tracking() {
        let repo = setup_test_repo().await;
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create execution that fails
        let mut execution = TaskExecution::new(task.id, "test-branch".to_string());
        execution.add_log("Starting execution".to_string());
        execution.complete_failure("Build failed: syntax error".to_string());
        
        repo.task_executions.create(&execution).await.unwrap();
        
        // Retrieve and verify failure
        let retrieved = repo.task_executions.get(execution.id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Failed);
        assert_eq!(retrieved.error_message, Some("Build failed: syntax error".to_string()));
        assert!(retrieved.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_execution_duration() {
        let repo = setup_test_repo().await;
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create execution
        let mut execution = TaskExecution::new(task.id, "test-branch".to_string());
        let start = execution.started_at;
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Complete execution
        execution.complete_success(None);
        
        // Check duration
        let duration = execution.duration().unwrap();
        assert!(duration.num_milliseconds() >= 100);
        
        repo.task_executions.create(&execution).await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_executions_different_tasks() {
        let repo = setup_test_repo().await;
        
        // Create multiple tasks
        let task1 = Task::new("Task 1".to_string(), "Description".to_string());
        let task2 = Task::new("Task 2".to_string(), "Description".to_string());
        repo.tasks.create(&task1).await.unwrap();
        repo.tasks.create(&task2).await.unwrap();
        
        // Create executions for each task
        let exec1_1 = TaskExecution::new(task1.id, "task1-branch1".to_string());
        let exec1_2 = TaskExecution::new(task1.id, "task1-branch2".to_string());
        let exec2_1 = TaskExecution::new(task2.id, "task2-branch1".to_string());
        
        repo.task_executions.create(&exec1_1).await.unwrap();
        repo.task_executions.create(&exec1_2).await.unwrap();
        repo.task_executions.create(&exec2_1).await.unwrap();
        
        // Verify executions are properly separated by task
        let task1_execs = repo.task_executions.list_for_task(task1.id).await.unwrap();
        let task2_execs = repo.task_executions.list_for_task(task2.id).await.unwrap();
        
        assert_eq!(task1_execs.len(), 2);
        assert_eq!(task2_execs.len(), 1);
        assert_eq!(task2_execs[0].branch_name, "task2-branch1");
    }
}