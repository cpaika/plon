use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::dependency::{Dependency, DependencyType, DependencyGraph};
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
        self.repository.dependencies.delete(from_task_id, to_task_id).await
    }

    pub async fn get_dependencies_for_task(&self, task_id: Uuid) -> Result<Vec<Dependency>> {
        self.repository.dependencies.get_dependencies_for_task(task_id).await
    }

    pub async fn get_dependents_for_task(&self, task_id: Uuid) -> Result<Vec<Dependency>> {
        self.repository.dependencies.get_dependents_for_task(task_id).await
    }

    pub async fn get_all_dependencies(&self) -> Result<Vec<Dependency>> {
        self.repository.dependencies.list_all().await
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

    pub async fn check_for_cycles(
        &self,
        from_task_id: Uuid,
        to_task_id: Uuid,
    ) -> Result<bool> {
        let mut graph = self.build_dependency_graph().await?;
        
        // Try adding the new dependency
        let test_dep = Dependency::new(from_task_id, to_task_id, DependencyType::FinishToStart);
        graph.add_dependency(&test_dep).map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(graph.has_cycle())
    }
}