#[cfg(test)]
mod tests {
    use super::super::goal::*;
    use super::super::task::{Task, TaskStatus};
    use chrono::{Utc, Duration};
    use uuid::Uuid;

    #[test]
    fn test_create_goal() {
        let goal = Goal::new(
            "Q1 2024 Objectives".to_string(),
            "Complete major project milestones for Q1".to_string(),
        );

        assert_eq!(goal.title, "Q1 2024 Objectives");
        assert_eq!(goal.description, "Complete major project milestones for Q1");
        assert_eq!(goal.status, GoalStatus::NotStarted);
    }

    #[test]
    fn test_goal_add_task() {
        let mut goal = Goal::new(
            "Release v1.0".to_string(),
            "Ship the first version".to_string(),
        );
        
        let task = Task::new(
            "Write documentation".to_string(),
            "Create user guide".to_string(),
        );
        
        goal.add_task(task.id);
        assert_eq!(goal.task_ids.len(), 1);
        assert!(goal.task_ids.contains(&task.id));
    }

    #[test]
    fn test_goal_progress_calculation() {
        let mut goal = Goal::new(
            "Feature Complete".to_string(),
            "Complete all features".to_string(),
        );
        
        // Create mock tasks
        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();
        let task3_id = Uuid::new_v4();
        
        goal.add_task(task1_id);
        goal.add_task(task2_id);
        goal.add_task(task3_id);
        
        // Create tasks
        let mut tasks = vec![
            {
                let mut t = Task::new("Task 1".to_string(), "".to_string());
                t.id = task1_id;
                t.status = TaskStatus::Done;
                t
            },
            {
                let mut t = Task::new("Task 2".to_string(), "".to_string());
                t.id = task2_id;
                t.status = TaskStatus::InProgress;
                t
            },
            {
                let mut t = Task::new("Task 3".to_string(), "".to_string());
                t.id = task3_id;
                t.status = TaskStatus::Todo;
                t
            },
        ];
        
        goal.update_progress(&tasks);
        assert!((goal.progress - 33.33).abs() < 1.0); // 1 out of 3 tasks done
    }

    #[test]
    fn test_goal_with_subtasks_progress() {
        let mut goal = Goal::new(
            "Complex Goal".to_string(),
            "Goal with subtasks".to_string(),
        );
        
        let task_id = Uuid::new_v4();
        goal.add_task(task_id);
        
        let mut task = Task::new("Task with subtasks".to_string(), "".to_string());
        task.id = task_id;
        task.status = TaskStatus::InProgress;
        
        // Add subtasks
        task.add_subtask("Subtask 1".to_string());
        task.add_subtask("Subtask 2".to_string());
        task.add_subtask("Subtask 3".to_string());
        
        // Complete one subtask
        task.subtasks[0].completed = true;
        
        let tasks = vec![task];
        goal.update_progress(&tasks);
        
        // Progress should reflect subtask completion
        assert!(goal.progress > 0.0 && goal.progress < 100.0);
    }

    #[test]
    fn test_goal_hierarchy() {
        let mut parent_goal = Goal::new(
            "Annual Goals".to_string(),
            "Company-wide objectives".to_string(),
        );
        
        let child_goal = Goal::new(
            "Q1 Goals".to_string(),
            "First quarter objectives".to_string(),
        );
        
        parent_goal.add_subgoal(child_goal.id);
        assert_eq!(parent_goal.subgoal_ids.len(), 1);
        assert!(parent_goal.subgoal_ids.contains(&child_goal.id));
    }

    #[test]
    fn test_goal_completion() {
        let mut goal = Goal::new(
            "Complete Project".to_string(),
            "Finish all project tasks".to_string(),
        );
        
        goal.target_date = Some(Utc::now() + Duration::days(30));
        
        assert!(!goal.is_overdue());
        assert!(!goal.is_complete());
        
        goal.progress = 100.0;
        goal.status = GoalStatus::Completed;
        goal.completed_at = Some(Utc::now());
        
        assert!(goal.is_complete());
    }

    #[test]
    fn test_goal_overdue() {
        let mut goal = Goal::new(
            "Urgent Goal".to_string(),
            "Should have been done yesterday".to_string(),
        );
        
        goal.target_date = Some(Utc::now() - Duration::days(1));
        goal.status = GoalStatus::Active;
        
        assert!(goal.is_overdue());
    }

    #[test]
    fn test_goal_estimated_hours() {
        let mut goal = Goal::new(
            "Timed Goal".to_string(),
            "Goal with time estimates".to_string(),
        );
        
        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();
        
        goal.add_task(task1_id);
        goal.add_task(task2_id);
        
        let tasks = vec![
            {
                let mut t = Task::new("Task 1".to_string(), "".to_string());
                t.id = task1_id;
                t.estimated_hours = Some(5.0);
                t.actual_hours = Some(4.0);
                t.status = TaskStatus::Done;
                t
            },
            {
                let mut t = Task::new("Task 2".to_string(), "".to_string());
                t.id = task2_id;
                t.estimated_hours = Some(3.0);
                t.status = TaskStatus::InProgress;
                t
            },
        ];
        
        let (estimated, actual) = goal.calculate_hours(&tasks);
        assert_eq!(estimated, 8.0);
        assert_eq!(actual, 4.0);
    }

    #[test]
    fn test_goal_status_transitions() {
        let mut goal = Goal::new(
            "Status Test".to_string(),
            "Test status changes".to_string(),
        );
        
        assert_eq!(goal.status, GoalStatus::Active);
        
        goal.status = GoalStatus::OnHold;
        assert_eq!(goal.status, GoalStatus::OnHold);
        
        goal.status = GoalStatus::Completed;
        goal.completed_at = Some(Utc::now());
        assert!(goal.is_complete());
        
        goal.status = GoalStatus::Cancelled;
        assert_eq!(goal.status, GoalStatus::Cancelled);
    }

    #[test]
    fn test_goal_color() {
        let mut goal = Goal::new(
            "Colorful Goal".to_string(),
            "Goal with custom color".to_string(),
        );
        
        goal.color = "#FF5733".to_string();
        assert_eq!(goal.color, "#FF5733");
    }

    #[test]
    fn test_goal_position() {
        let mut goal = Goal::new(
            "Positioned Goal".to_string(),
            "Goal with map position".to_string(),
        );
        
        goal.position_x = 100.0;
        goal.position_y = 200.0;
        goal.position_width = 300.0;
        goal.position_height = 150.0;
        
        assert_eq!(goal.position_x, 100.0);
        assert_eq!(goal.position_y, 200.0);
        assert_eq!(goal.position_width, 300.0);
        assert_eq!(goal.position_height, 150.0);
    }
}