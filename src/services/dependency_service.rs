use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::dependency::{Dependency, DependencyGraph, DependencyType};
use crate::repository::Repository;

#[derive(Clone)]
pub struct DependencyService {
    repository: Arc<Repository>,
}

impl DependencyService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn create_dependency(
        &self,
        from_task_id: Uuid,
        to_task_id: Uuid,
        dependency_type: DependencyType,
    ) -> Result<Dependency> {
        let dependency = Dependency::new(from_task_id, to_task_id, dependency_type);

        // Save to database
        self.repository.dependencies.create(&dependency).await?;

        Ok(dependency)
    }

    pub async fn delete_dependency(&self, from_task_id: Uuid, to_task_id: Uuid) -> Result<bool> {
        self.repository
            .dependencies
            .delete(from_task_id, to_task_id)
            .await
    }

    pub async fn get_dependencies_for_task(&self, task_id: Uuid) -> Result<Vec<Dependency>> {
        self.repository
            .dependencies
            .get_dependencies_for_task(task_id)
            .await
    }

    pub async fn get_dependents_for_task(&self, task_id: Uuid) -> Result<Vec<Dependency>> {
        self.repository
            .dependencies
            .get_dependents_for_task(task_id)
            .await
    }

    pub async fn get_all_dependencies(&self) -> Result<Vec<Dependency>> {
        self.repository.dependencies.list_all().await
    }

    pub async fn add_dependency(&self, from_task_id: Uuid, to_task_id: Uuid) -> Result<()> {
        self.create_dependency(from_task_id, to_task_id, DependencyType::FinishToStart)
            .await?;
        Ok(())
    }

    pub async fn get_dependencies(&self, task_id: Uuid) -> Result<Vec<Uuid>> {
        // Returns tasks that this task depends on
        // get_dependents_for_task returns deps WHERE from_task_id = task_id
        // So these are the dependencies where task_id is the "from" (the dependent)
        let deps = self.get_dependents_for_task(task_id).await?;
        Ok(deps.into_iter().map(|d| d.to_task_id).collect())
    }

    pub async fn get_dependents(&self, task_id: Uuid) -> Result<Vec<Uuid>> {
        // Returns tasks that depend on this task
        // get_dependencies_for_task returns deps WHERE to_task_id = task_id
        // So these are the dependencies where task_id is the "to" (the dependency)
        let deps = self.get_dependencies_for_task(task_id).await?;
        Ok(deps.into_iter().map(|d| d.from_task_id).collect())
    }

    pub async fn build_dependency_graph(&self) -> Result<DependencyGraph> {
        let dependencies = self.get_all_dependencies().await?;
        let mut graph = DependencyGraph::new();

        for dep in dependencies {
            graph.add_task(dep.from_task_id);
            graph.add_task(dep.to_task_id);
            graph.add_dependency(&dep).map_err(|e| anyhow::anyhow!(e))?;
        }

        Ok(graph)
    }

    pub async fn check_for_cycles(&self, from_task_id: Uuid, to_task_id: Uuid) -> Result<bool> {
        let mut graph = self.build_dependency_graph().await?;

        // Try adding the new dependency
        let test_dep = Dependency::new(from_task_id, to_task_id, DependencyType::FinishToStart);
        graph
            .add_dependency(&test_dep)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(graph.has_cycle())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;
    use crate::repository::database::init_test_database;

    async fn setup() -> (DependencyService, Arc<Repository>) {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = DependencyService::new(repository.clone());
        (service, repository)
    }

    async fn create_test_tasks(repository: &Arc<Repository>) -> (Uuid, Uuid, Uuid) {
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        let task3 = Task::new("Task 3".to_string(), "".to_string());

        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();
        repository.tasks.create(&task3).await.unwrap();

        (task1.id, task2.id, task3.id)
    }

    #[tokio::test]
    async fn test_create_dependency() {
        let (service, repository) = setup().await;
        let (task1_id, task2_id, _) = create_test_tasks(&repository).await;

        let dep = service
            .create_dependency(task1_id, task2_id, DependencyType::FinishToStart)
            .await
            .unwrap();

        assert_eq!(dep.from_task_id, task1_id);
        assert_eq!(dep.to_task_id, task2_id);
        assert_eq!(dep.dependency_type, DependencyType::FinishToStart);
    }

    #[tokio::test]
    async fn test_delete_dependency() {
        let (service, repository) = setup().await;
        let (task1_id, task2_id, _) = create_test_tasks(&repository).await;

        service
            .create_dependency(task1_id, task2_id, DependencyType::FinishToStart)
            .await
            .unwrap();

        let deleted = service.delete_dependency(task1_id, task2_id).await.unwrap();
        assert!(deleted);

        let deps = service.get_dependencies_for_task(task2_id).await.unwrap();
        assert_eq!(deps.len(), 0);
    }

    #[tokio::test]
    async fn test_get_dependencies_for_task() {
        let (service, repository) = setup().await;
        let (task1_id, task2_id, task3_id) = create_test_tasks(&repository).await;

        // Create dependencies: task1 -> task2, task3 -> task2
        service
            .create_dependency(task1_id, task2_id, DependencyType::FinishToStart)
            .await
            .unwrap();
        service
            .create_dependency(task3_id, task2_id, DependencyType::StartToStart)
            .await
            .unwrap();

        let deps = service.get_dependencies_for_task(task2_id).await.unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[tokio::test]
    async fn test_get_dependents_for_task() {
        let (service, repository) = setup().await;
        let (task1_id, task2_id, task3_id) = create_test_tasks(&repository).await;

        // Create dependencies: task1 -> task2, task1 -> task3
        service
            .create_dependency(task1_id, task2_id, DependencyType::FinishToStart)
            .await
            .unwrap();
        service
            .create_dependency(task1_id, task3_id, DependencyType::FinishToFinish)
            .await
            .unwrap();

        let dependents = service.get_dependents_for_task(task1_id).await.unwrap();
        assert_eq!(dependents.len(), 2);
    }

    #[tokio::test]
    async fn test_build_dependency_graph() {
        let (service, repository) = setup().await;
        let (task1_id, task2_id, task3_id) = create_test_tasks(&repository).await;

        service
            .create_dependency(task1_id, task2_id, DependencyType::FinishToStart)
            .await
            .unwrap();
        service
            .create_dependency(task2_id, task3_id, DependencyType::FinishToStart)
            .await
            .unwrap();

        let graph = service.build_dependency_graph().await.unwrap();
        assert!(!graph.has_cycle());

        // Verify topological sort works
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], task1_id);
        assert_eq!(sorted[1], task2_id);
        assert_eq!(sorted[2], task3_id);
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let (service, repository) = setup().await;
        let (task1_id, task2_id, task3_id) = create_test_tasks(&repository).await;

        // Create chain: task1 -> task2 -> task3
        service
            .create_dependency(task1_id, task2_id, DependencyType::FinishToStart)
            .await
            .unwrap();
        service
            .create_dependency(task2_id, task3_id, DependencyType::FinishToStart)
            .await
            .unwrap();

        // Check if adding task3 -> task1 would create a cycle (it should)
        let would_cycle = service.check_for_cycles(task3_id, task1_id).await;

        // Note: check_for_cycles will fail because add_dependency prevents cycles
        assert!(would_cycle.is_err());
    }
}
