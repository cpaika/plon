pub mod database;
pub mod task_repository;
pub mod goal_repository;
pub mod resource_repository;
pub mod comment_repository;
pub mod dependency_repository;
pub mod recurring_repository;
pub mod task_config_repository;

use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct Repository {
    pub pool: Arc<SqlitePool>,
    pub tasks: task_repository::TaskRepository,
    pub goals: goal_repository::GoalRepository,
    pub resources: resource_repository::ResourceRepository,
    pub comments: comment_repository::CommentRepository,
    pub dependencies: dependency_repository::DependencyRepository,
    pub recurring: recurring_repository::RecurringRepository,
    pub task_configs: task_config_repository::TaskConfigRepository,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        let pool = Arc::new(pool);
        Self {
            tasks: task_repository::TaskRepository::new(pool.clone()),
            goals: goal_repository::GoalRepository::new(pool.clone()),
            resources: resource_repository::ResourceRepository::new(pool.clone()),
            comments: comment_repository::CommentRepository::new(pool.clone()),
            dependencies: dependency_repository::DependencyRepository::new(pool.clone()),
            recurring: recurring_repository::RecurringRepository::new(pool.clone()),
            task_configs: task_config_repository::TaskConfigRepository::new(pool.clone()),
            pool,
        }
    }
}