#[cfg(test)]
mod task_execution_tests {
    use plon::domain::task_execution::{TaskExecution, ExecutionStatus};
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_create_task_execution() {
        let task_id = Uuid::new_v4();
        let branch_name = "task/123-test-task".to_string();
        
        let execution = TaskExecution::new(task_id, branch_name.clone());
        
        assert_eq!(execution.task_id, task_id);
        assert_eq!(execution.branch_name, branch_name);
        assert_eq!(execution.status, ExecutionStatus::Running);
        assert!(execution.completed_at.is_none());
        assert!(execution.pr_url.is_none());
        assert!(execution.error_message.is_none());
        assert!(execution.output_log.is_empty());
    }

    #[test]
    fn test_complete_success() {
        let task_id = Uuid::new_v4();
        let mut execution = TaskExecution::new(task_id, "branch".to_string());
        
        let pr_url = Some("https://github.com/user/repo/pull/1".to_string());
        execution.complete_success(pr_url.clone());
        
        assert_eq!(execution.status, ExecutionStatus::Success);
        assert!(execution.completed_at.is_some());
        assert_eq!(execution.pr_url, pr_url);
        assert!(execution.error_message.is_none());
    }

    #[test]
    fn test_complete_failure() {
        let task_id = Uuid::new_v4();
        let mut execution = TaskExecution::new(task_id, "branch".to_string());
        
        let error = "Test error".to_string();
        execution.complete_failure(error.clone());
        
        assert_eq!(execution.status, ExecutionStatus::Failed);
        assert!(execution.completed_at.is_some());
        assert_eq!(execution.error_message, Some(error));
        assert!(execution.pr_url.is_none());
    }

    #[test]
    fn test_add_log() {
        let task_id = Uuid::new_v4();
        let mut execution = TaskExecution::new(task_id, "branch".to_string());
        
        execution.add_log("First log".to_string());
        execution.add_log("Second log".to_string());
        
        assert_eq!(execution.output_log.len(), 2);
        assert!(execution.output_log[0].contains("First log"));
        assert!(execution.output_log[1].contains("Second log"));
    }

    #[test]
    fn test_is_active() {
        let task_id = Uuid::new_v4();
        let mut execution = TaskExecution::new(task_id, "branch".to_string());
        
        // Running status is active
        assert!(execution.is_active());
        
        // PendingReview is active
        execution.status = ExecutionStatus::PendingReview;
        assert!(execution.is_active());
        
        // Success is not active
        execution.status = ExecutionStatus::Success;
        assert!(!execution.is_active());
        
        // Failed is not active
        execution.status = ExecutionStatus::Failed;
        assert!(!execution.is_active());
    }
}