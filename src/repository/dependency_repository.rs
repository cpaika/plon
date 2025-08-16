use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::dependency::{Dependency, DependencyType, DependencyGraph};

#[derive(Clone)]
pub struct DependencyRepository {
    pool: Arc<SqlitePool>,
}

impl DependencyRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dependency: &Dependency) -> Result<()> {
        let id = dependency.id.to_string();
        let from_task_id = dependency.from_task_id.to_string();
        let to_task_id = dependency.to_task_id.to_string();
        let dependency_type = dependency_type_to_string(&dependency.dependency_type);
        let created_at = dependency.created_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO dependencies (id, from_task_id, to_task_id, dependency_type, created_at)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(from_task_id)
        .bind(to_task_id)
        .bind(dependency_type)
        .bind(created_at)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, from_task_id: Uuid, to_task_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM dependencies WHERE from_task_id = ? AND to_task_id = ?"
        )
        .bind(from_task_id.to_string())
        .bind(to_task_id.to_string())
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dependencies_for_task(&self, task_id: Uuid) -> Result<Vec<Dependency>> {
        let rows = sqlx::query(
            "SELECT id, from_task_id, to_task_id, dependency_type, created_at
             FROM dependencies WHERE to_task_id = ?"
        )
        .bind(task_id.to_string())
        .fetch_all(&*self.pool)
        .await?;

        let mut dependencies = Vec::new();
        for row in rows {
            dependencies.push(row_to_dependency(row)?);
        }

        Ok(dependencies)
    }

    pub async fn get_dependents_for_task(&self, task_id: Uuid) -> Result<Vec<Dependency>> {
        let rows = sqlx::query(
            "SELECT id, from_task_id, to_task_id, dependency_type, created_at
             FROM dependencies WHERE from_task_id = ?"
        )
        .bind(task_id.to_string())
        .fetch_all(&*self.pool)
        .await?;

        let mut dependencies = Vec::new();
        for row in rows {
            dependencies.push(row_to_dependency(row)?);
        }

        Ok(dependencies)
    }

    pub async fn list_all(&self) -> Result<Vec<Dependency>> {
        let rows = sqlx::query(
            "SELECT id, from_task_id, to_task_id, dependency_type, created_at
             FROM dependencies"
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut dependencies = Vec::new();
        for row in rows {
            dependencies.push(row_to_dependency(row)?);
        }

        Ok(dependencies)
    }

    pub async fn get_graph(&self) -> Result<DependencyGraph> {
        let dependencies = self.list_all().await?;
        let mut graph = DependencyGraph::new();
        
        // Add all tasks to the graph
        for dep in &dependencies {
            graph.add_task(dep.from_task_id);
            graph.add_task(dep.to_task_id);
        }
        
        // Add all dependencies
        for dep in dependencies {
            graph.add_dependency(&dep).map_err(|e| anyhow::anyhow!(e))?;
        }
        
        Ok(graph)
    }
}

fn dependency_type_to_string(dep_type: &DependencyType) -> &'static str {
    match dep_type {
        DependencyType::FinishToStart => "FinishToStart",
        DependencyType::StartToStart => "StartToStart",
        DependencyType::FinishToFinish => "FinishToFinish",
        DependencyType::StartToFinish => "StartToFinish",
    }
}

fn string_to_dependency_type(s: &str) -> Result<DependencyType> {
    match s {
        "FinishToStart" => Ok(DependencyType::FinishToStart),
        "StartToStart" => Ok(DependencyType::StartToStart),
        "FinishToFinish" => Ok(DependencyType::FinishToFinish),
        "StartToFinish" => Ok(DependencyType::StartToFinish),
        _ => Err(anyhow::anyhow!("Invalid dependency type: {}", s)),
    }
}

fn row_to_dependency(row: sqlx::sqlite::SqliteRow) -> Result<Dependency> {
    let id = Uuid::parse_str(row.get("id"))?;
    let from_task_id = Uuid::parse_str(row.get("from_task_id"))?;
    let to_task_id = Uuid::parse_str(row.get("to_task_id"))?;
    let dependency_type = string_to_dependency_type(row.get("dependency_type"))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&Utc);
    
    Ok(Dependency {
        id,
        from_task_id,
        to_task_id,
        dependency_type,
        created_at,
    })
}