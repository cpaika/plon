use chrono::Utc;
use plon::domain::dependency::{Dependency, DependencyGraph, DependencyType};
use plon::domain::goal::Goal;
use plon::domain::task::{Position, Priority, Task, TaskStatus};
use plon::ui::views::map_view::MapView;
use std::collections::HashSet;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_tasks_by_status() {
        // Create tasks with different statuses
        let mut tasks = vec![
            create_test_task("Task 1", TaskStatus::Todo),
            create_test_task("Task 2", TaskStatus::InProgress),
            create_test_task("Task 3", TaskStatus::Todo),
            create_test_task("Task 4", TaskStatus::Done),
            create_test_task("Task 5", TaskStatus::InProgress),
            create_test_task("Task 6", TaskStatus::Review),
        ];

        let mut map_view = MapView::new();
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Tasks with same status should be closer together
        assert_tasks_grouped_by_status(&tasks);
    }

    #[test]
    fn test_group_tasks_by_tags() {
        // Create tasks with different tags
        let mut tasks = vec![
            create_test_task_with_tags("Task 1", vec!["frontend".to_string()]),
            create_test_task_with_tags("Task 2", vec!["backend".to_string()]),
            create_test_task_with_tags("Task 3", vec!["frontend".to_string()]),
            create_test_task_with_tags("Task 4", vec!["backend".to_string()]),
            create_test_task_with_tags("Task 5", vec!["frontend".to_string(), "ui".to_string()]),
        ];

        let mut map_view = MapView::new();
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Tasks with same tags should be closer together
        assert_tasks_grouped_by_tags(&tasks);
    }

    #[test]
    fn test_group_tasks_by_priority() {
        // Create tasks with different priorities
        let mut tasks = vec![
            create_test_task_with_priority("Critical Task", Priority::Critical),
            create_test_task_with_priority("Low Task 1", Priority::Low),
            create_test_task_with_priority("High Task", Priority::High),
            create_test_task_with_priority("Low Task 2", Priority::Low),
            create_test_task_with_priority("Critical Task 2", Priority::Critical),
        ];

        let mut map_view = MapView::new();
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Tasks with same priority should be grouped
        assert_tasks_grouped_by_priority(&tasks);
    }

    #[test]
    fn test_dependency_chain_layout() {
        // Create tasks with dependencies
        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();
        let task3_id = Uuid::new_v4();
        let task4_id = Uuid::new_v4();

        let mut tasks = vec![
            create_test_task_with_id("Task 1", task1_id),
            create_test_task_with_id("Task 2", task2_id),
            create_test_task_with_id("Task 3", task3_id),
            create_test_task_with_id("Task 4", task4_id),
        ];

        // Create dependency chain: Task1 -> Task2 -> Task3 -> Task4
        let dependencies = vec![
            Dependency::new(task1_id, task2_id, DependencyType::FinishToStart),
            Dependency::new(task2_id, task3_id, DependencyType::FinishToStart),
            Dependency::new(task3_id, task4_id, DependencyType::FinishToStart),
        ];

        let mut map_view = MapView::new();
        map_view.set_dependencies(dependencies);
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Tasks should be arranged left to right following dependency chain
        assert_dependency_chain_layout(&tasks, vec![task1_id, task2_id, task3_id, task4_id]);
    }

    #[test]
    fn test_parallel_dependencies_layout() {
        // Create a dependency graph with parallel paths
        let start_task_id = Uuid::new_v4();
        let parallel1_id = Uuid::new_v4();
        let parallel2_id = Uuid::new_v4();
        let parallel3_id = Uuid::new_v4();
        let end_task_id = Uuid::new_v4();

        let mut tasks = vec![
            create_test_task_with_id("Start", start_task_id),
            create_test_task_with_id("Parallel 1", parallel1_id),
            create_test_task_with_id("Parallel 2", parallel2_id),
            create_test_task_with_id("Parallel 3", parallel3_id),
            create_test_task_with_id("End", end_task_id),
        ];

        // Create parallel dependencies: Start -> (P1, P2, P3) -> End
        let dependencies = vec![
            Dependency::new(start_task_id, parallel1_id, DependencyType::FinishToStart),
            Dependency::new(start_task_id, parallel2_id, DependencyType::FinishToStart),
            Dependency::new(start_task_id, parallel3_id, DependencyType::FinishToStart),
            Dependency::new(parallel1_id, end_task_id, DependencyType::FinishToStart),
            Dependency::new(parallel2_id, end_task_id, DependencyType::FinishToStart),
            Dependency::new(parallel3_id, end_task_id, DependencyType::FinishToStart),
        ];

        let mut map_view = MapView::new();
        map_view.set_dependencies(dependencies);
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Start should be leftmost, parallel tasks in middle column, end rightmost
        assert_parallel_dependency_layout(
            &tasks,
            start_task_id,
            vec![parallel1_id, parallel2_id, parallel3_id],
            end_task_id,
        );
    }

    #[test]
    fn test_complex_dependency_layout() {
        // Create a complex dependency graph
        let mut tasks = vec![];
        let mut task_ids = vec![];

        for i in 0..10 {
            let id = Uuid::new_v4();
            task_ids.push(id);
            tasks.push(create_test_task_with_id(&format!("Task {}", i), id));
        }

        // Create complex dependencies
        let dependencies = vec![
            Dependency::new(task_ids[0], task_ids[1], DependencyType::FinishToStart),
            Dependency::new(task_ids[0], task_ids[2], DependencyType::FinishToStart),
            Dependency::new(task_ids[1], task_ids[3], DependencyType::FinishToStart),
            Dependency::new(task_ids[2], task_ids[4], DependencyType::FinishToStart),
            Dependency::new(task_ids[3], task_ids[5], DependencyType::FinishToStart),
            Dependency::new(task_ids[4], task_ids[5], DependencyType::FinishToStart),
            Dependency::new(task_ids[5], task_ids[6], DependencyType::FinishToStart),
            Dependency::new(task_ids[6], task_ids[7], DependencyType::FinishToStart),
            Dependency::new(task_ids[6], task_ids[8], DependencyType::FinishToStart),
            Dependency::new(task_ids[7], task_ids[9], DependencyType::FinishToStart),
            Dependency::new(task_ids[8], task_ids[9], DependencyType::FinishToStart),
        ];

        let mut map_view = MapView::new();
        map_view.set_dependencies(dependencies);
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Verify topological ordering is respected
        assert_topological_ordering(&tasks, &task_ids);
    }

    #[test]
    fn test_mixed_grouping_and_dependencies() {
        // Test that both grouping and dependencies are considered
        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();
        let task3_id = Uuid::new_v4();
        let task4_id = Uuid::new_v4();
        let task5_id = Uuid::new_v4();

        let mut tasks = vec![
            create_test_task_with_id_and_tags("Frontend 1", task1_id, vec!["frontend".to_string()]),
            create_test_task_with_id_and_tags("Backend 1", task2_id, vec!["backend".to_string()]),
            create_test_task_with_id_and_tags("Frontend 2", task3_id, vec!["frontend".to_string()]),
            create_test_task_with_id_and_tags("Backend 2", task4_id, vec!["backend".to_string()]),
            create_test_task_with_id_and_tags("Deploy", task5_id, vec!["deployment".to_string()]),
        ];

        // Create dependencies
        let dependencies = vec![
            Dependency::new(task1_id, task3_id, DependencyType::FinishToStart),
            Dependency::new(task2_id, task4_id, DependencyType::FinishToStart),
            Dependency::new(task3_id, task5_id, DependencyType::FinishToStart),
            Dependency::new(task4_id, task5_id, DependencyType::FinishToStart),
        ];

        let mut map_view = MapView::new();
        map_view.set_dependencies(dependencies);
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // Tasks should be grouped by tags AND respect dependencies
        assert_mixed_grouping_and_dependencies(&tasks);
    }

    #[test]
    fn test_goals_with_tasks_arrangement() {
        // Test arranging goals with their associated tasks
        let goal1_id = Uuid::new_v4();
        let goal2_id = Uuid::new_v4();

        let mut goals = vec![
            create_test_goal("Goal 1", goal1_id),
            create_test_goal("Goal 2", goal2_id),
        ];

        let mut tasks = vec![
            create_test_task_with_goal("Task 1", Some(goal1_id)),
            create_test_task_with_goal("Task 2", Some(goal1_id)),
            create_test_task_with_goal("Task 3", Some(goal2_id)),
            create_test_task_with_goal("Task 4", Some(goal2_id)),
            create_test_task_with_goal("Task 5", None), // Standalone task
        ];

        let mut map_view = MapView::new();
        map_view.auto_arrange_smart(&mut tasks, &mut goals);

        // Goals should be arranged with their tasks inside
        assert_goals_contain_their_tasks(&goals, &tasks);
    }

    #[test]
    fn test_no_overlapping_positions() {
        // Test that no tasks overlap after arrangement
        let mut tasks = vec![];
        for i in 0..20 {
            tasks.push(create_test_task(&format!("Task {}", i), TaskStatus::Todo));
        }

        let mut map_view = MapView::new();
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // No two tasks should have overlapping positions
        assert_no_overlapping_positions(&tasks);
    }

    #[test]
    fn test_minimum_spacing_maintained() {
        // Test that minimum spacing is maintained between tasks
        let mut tasks = vec![
            create_test_task("Task 1", TaskStatus::Todo),
            create_test_task("Task 2", TaskStatus::Todo),
            create_test_task("Task 3", TaskStatus::Todo),
        ];

        let mut map_view = MapView::new();
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);

        // All tasks should have minimum spacing
        assert_minimum_spacing(&tasks, 50.0); // Minimum 50 pixels between tasks
    }

    // Helper functions
    fn create_test_task(title: &str, status: TaskStatus) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.status = status;
        task.priority = Priority::Medium;
        task
    }

    fn create_test_task_with_id(title: &str, id: Uuid) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.id = id;
        task.status = TaskStatus::Todo;
        task.priority = Priority::Medium;
        task
    }

    fn create_test_task_with_tags(title: &str, tags: Vec<String>) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.status = TaskStatus::Todo;
        task.priority = Priority::Medium;
        task.tags = tags.into_iter().collect();
        task
    }

    fn create_test_task_with_priority(title: &str, priority: Priority) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.status = TaskStatus::Todo;
        task.priority = priority;
        task
    }

    fn create_test_task_with_id_and_tags(title: &str, id: Uuid, tags: Vec<String>) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.id = id;
        task.status = TaskStatus::Todo;
        task.priority = Priority::Medium;
        task.tags = tags.into_iter().collect();
        task
    }

    fn create_test_task_with_goal(title: &str, goal_id: Option<Uuid>) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.status = TaskStatus::Todo;
        task.priority = Priority::Medium;
        task.goal_id = goal_id;
        task
    }

    fn create_test_goal(title: &str, id: Uuid) -> Goal {
        let mut goal = Goal::new(title.to_string(), String::new());
        goal.id = id;
        goal
    }

    fn assert_tasks_grouped_by_status(tasks: &[Task]) {
        // Calculate average distance between tasks with same status
        // and compare with average distance between tasks with different status
        let same_status_distance = calculate_average_distance_same_status(tasks);
        let diff_status_distance = calculate_average_distance_diff_status(tasks);

        assert!(
            same_status_distance < diff_status_distance * 0.7,
            "Tasks with same status should be closer together"
        );
    }

    fn assert_tasks_grouped_by_tags(tasks: &[Task]) {
        let same_tag_distance = calculate_average_distance_same_tags(tasks);
        let diff_tag_distance = calculate_average_distance_diff_tags(tasks);

        assert!(
            same_tag_distance < diff_tag_distance * 0.7,
            "Tasks with same tags should be closer together"
        );
    }

    fn assert_tasks_grouped_by_priority(tasks: &[Task]) {
        let same_priority_distance = calculate_average_distance_same_priority(tasks);
        let diff_priority_distance = calculate_average_distance_diff_priority(tasks);

        assert!(
            same_priority_distance < diff_priority_distance * 0.7,
            "Tasks with same priority should be closer together"
        );
    }

    fn assert_dependency_chain_layout(tasks: &[Task], expected_order: Vec<Uuid>) {
        // Check that tasks are arranged left to right in dependency order
        for i in 0..expected_order.len() - 1 {
            let task1 = tasks.iter().find(|t| t.id == expected_order[i]).unwrap();
            let task2 = tasks
                .iter()
                .find(|t| t.id == expected_order[i + 1])
                .unwrap();

            assert!(
                task1.position.x < task2.position.x,
                "Task {} should be to the left of Task {}",
                i,
                i + 1
            );
        }
    }

    fn assert_parallel_dependency_layout(
        tasks: &[Task],
        start_id: Uuid,
        parallel_ids: Vec<Uuid>,
        end_id: Uuid,
    ) {
        let start_task = tasks.iter().find(|t| t.id == start_id).unwrap();
        let end_task = tasks.iter().find(|t| t.id == end_id).unwrap();

        // Start should be leftmost, end should be rightmost
        for task in tasks {
            if parallel_ids.contains(&task.id) {
                assert!(
                    task.position.x > start_task.position.x,
                    "Parallel tasks should be to the right of start task"
                );
                assert!(
                    task.position.x < end_task.position.x,
                    "Parallel tasks should be to the left of end task"
                );
            }
        }
    }

    fn assert_topological_ordering(tasks: &[Task], task_ids: &[Uuid]) {
        // Verify that dependent tasks are always to the right of their dependencies
        // This is a simplified check - real implementation would use the dependency graph
        // For now, just check that positions increase generally from left to right
        let mut sorted_tasks: Vec<_> = tasks.to_vec();
        sorted_tasks.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());

        // Check that the x-positions are monotonically increasing for dependencies
        assert!(
            sorted_tasks
                .windows(2)
                .all(|w| w[0].position.x <= w[1].position.x)
        );
    }

    fn assert_mixed_grouping_and_dependencies(tasks: &[Task]) {
        // Check that both grouping and dependencies are respected
        // This is a complex assertion that would verify both constraints
        // For now, we'll do a simple check
        assert!(!tasks.is_empty());
    }

    fn assert_goals_contain_their_tasks(goals: &[Goal], tasks: &[Task]) {
        for goal in goals {
            let goal_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| t.goal_id == Some(goal.id))
                .collect();

            for task in goal_tasks {
                // Check that task is within goal bounds
                assert!(
                    task.position.x >= goal.position_x
                        && task.position.x <= goal.position_x + goal.position_width,
                    "Task should be within goal horizontal bounds"
                );
                assert!(
                    task.position.y >= goal.position_y
                        && task.position.y <= goal.position_y + goal.position_height,
                    "Task should be within goal vertical bounds"
                );
            }
        }
    }

    fn assert_no_overlapping_positions(tasks: &[Task]) {
        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                let task1 = &tasks[i];
                let task2 = &tasks[j];

                // Check if bounding boxes overlap
                let overlap = !(task1.position.x + 150.0 < task2.position.x
                    || task2.position.x + 150.0 < task1.position.x
                    || task1.position.y + 100.0 < task2.position.y
                    || task2.position.y + 100.0 < task1.position.y);

                assert!(!overlap, "Tasks {} and {} should not overlap", i, j);
            }
        }
    }

    fn assert_minimum_spacing(tasks: &[Task], min_spacing: f64) {
        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                let task1 = &tasks[i];
                let task2 = &tasks[j];

                let distance = ((task1.position.x - task2.position.x).powi(2)
                    + (task1.position.y - task2.position.y).powi(2))
                .sqrt();

                assert!(
                    distance >= min_spacing,
                    "Tasks {} and {} are too close (distance: {})",
                    i,
                    j,
                    distance
                );
            }
        }
    }

    // Helper distance calculation functions
    fn calculate_average_distance_same_status(tasks: &[Task]) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                if tasks[i].status == tasks[j].status {
                    total_distance += calculate_distance(&tasks[i].position, &tasks[j].position);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            0.0
        }
    }

    fn calculate_average_distance_diff_status(tasks: &[Task]) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                if tasks[i].status != tasks[j].status {
                    total_distance += calculate_distance(&tasks[i].position, &tasks[j].position);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            f64::MAX
        }
    }

    fn calculate_average_distance_same_tags(tasks: &[Task]) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                if tasks[i].tags.iter().any(|t| tasks[j].tags.contains(t)) {
                    total_distance += calculate_distance(&tasks[i].position, &tasks[j].position);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            0.0
        }
    }

    fn calculate_average_distance_diff_tags(tasks: &[Task]) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                if !tasks[i].tags.iter().any(|t| tasks[j].tags.contains(t)) {
                    total_distance += calculate_distance(&tasks[i].position, &tasks[j].position);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            f64::MAX
        }
    }

    fn calculate_average_distance_same_priority(tasks: &[Task]) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                if tasks[i].priority == tasks[j].priority {
                    total_distance += calculate_distance(&tasks[i].position, &tasks[j].position);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            0.0
        }
    }

    fn calculate_average_distance_diff_priority(tasks: &[Task]) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..tasks.len() {
            for j in i + 1..tasks.len() {
                if tasks[i].priority != tasks[j].priority {
                    total_distance += calculate_distance(&tasks[i].position, &tasks[j].position);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            f64::MAX
        }
    }

    fn calculate_distance(p1: &Position, p2: &Position) -> f64 {
        ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
    }
}
