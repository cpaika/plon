use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use crate::domain::task::{Task, TaskStatus};
use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use csv::Writer;
use std::io::Write;

pub struct ExportService {
    repository: Arc<Repository>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportedTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub tags: Vec<String>,
    pub assignee: Option<String>,
}

impl From<Task> for ExportedTask {
    fn from(task: Task) -> Self {
        ExportedTask {
            id: task.id.to_string(),
            title: task.title,
            description: task.description,
            status: format!("{:?}", task.status),
            priority: format!("{:?}", task.priority),
            created_at: task.created_at,
            updated_at: task.updated_at,
            due_date: task.due_date,
            estimated_hours: task.estimated_hours,
            actual_hours: task.actual_hours,
            tags: task.tags.iter().cloned().collect(),
            assignee: task.assignee,
        }
    }
}

impl ExportService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }
    
    /// Export tasks to JSON format
    pub async fn export_to_json(&self, filters: TaskFilters) -> Result<String> {
        let tasks = self.repository.tasks.list(filters).await?;
        let exported_tasks: Vec<ExportedTask> = tasks.into_iter().map(Into::into).collect();
        
        let json = serde_json::to_string_pretty(&exported_tasks)?;
        Ok(json)
    }
    
    /// Export tasks to CSV format
    pub async fn export_to_csv(&self, filters: TaskFilters) -> Result<String> {
        let tasks = self.repository.tasks.list(filters).await?;
        
        let mut wtr = Writer::from_writer(vec![]);
        
        // Write headers
        wtr.write_record(&[
            "ID",
            "Title",
            "Description",
            "Status",
            "Priority",
            "Created At",
            "Updated At",
            "Due Date",
            "Estimated Hours",
            "Actual Hours",
            "Tags",
            "Assignee",
        ])?;
        
        // Write task data
        for task in tasks {
            wtr.write_record(&[
                task.id.to_string(),
                task.title,
                task.description,
                format!("{:?}", task.status),
                format!("{:?}", task.priority),
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.due_date.map(|d| d.to_rfc3339()).unwrap_or_default(),
                task.estimated_hours.map(|h| h.to_string()).unwrap_or_default(),
                task.actual_hours.map(|h| h.to_string()).unwrap_or_default(),
                task.tags.iter().cloned().collect::<Vec<_>>().join(", "),
                task.assignee.unwrap_or_default(),
            ])?;
        }
        
        let data = wtr.into_inner()?;
        Ok(String::from_utf8(data)?)
    }
    
    /// Export tasks to Markdown format
    pub async fn export_to_markdown(&self, filters: TaskFilters) -> Result<String> {
        let tasks = self.repository.tasks.list(filters).await?;
        let mut output = String::new();
        
        output.push_str("# Task Export\n\n");
        output.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        // Group tasks by status
        let mut by_status: std::collections::HashMap<TaskStatus, Vec<Task>> = std::collections::HashMap::new();
        for task in tasks {
            by_status.entry(task.status).or_insert_with(Vec::new).push(task);
        }
        
        // Write sections for each status
        for status in &[TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Review, TaskStatus::Blocked, TaskStatus::Done] {
            if let Some(tasks) = by_status.get(status) {
                output.push_str(&format!("\n## {} ({})\n\n", status_to_string(*status), tasks.len()));
                
                for task in tasks {
                    output.push_str(&format!("### {}\n", task.title));
                    
                    if !task.description.is_empty() {
                        output.push_str(&format!("{}\n", task.description));
                    }
                    
                    output.push_str(&format!("- **Priority:** {:?}\n", task.priority));
                    
                    if let Some(due) = task.due_date {
                        output.push_str(&format!("- **Due:** {}\n", due.format("%Y-%m-%d")));
                    }
                    
                    if let Some(hours) = task.estimated_hours {
                        output.push_str(&format!("- **Estimated:** {} hours\n", hours));
                    }
                    
                    if let Some(hours) = task.actual_hours {
                        output.push_str(&format!("- **Actual:** {} hours\n", hours));
                    }
                    
                    if !task.tags.is_empty() {
                        output.push_str(&format!("- **Tags:** {}\n", task.tags.iter().cloned().collect::<Vec<_>>().join(", ")));
                    }
                    
                    if let Some(assignee) = &task.assignee {
                        output.push_str(&format!("- **Assignee:** {}\n", assignee));
                    }
                    
                    output.push_str("\n");
                }
            }
        }
        
        Ok(output)
    }
    
    /// Save export to file
    pub async fn export_to_file(&self, filters: TaskFilters, format: ExportFormat, path: &str) -> Result<()> {
        let content = match format {
            ExportFormat::Json => self.export_to_json(filters).await?,
            ExportFormat::Csv => self.export_to_csv(filters).await?,
            ExportFormat::Markdown => self.export_to_markdown(filters).await?,
        };
        
        let mut file = std::fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
}

fn status_to_string(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "To Do",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
        TaskStatus::Blocked => "Blocked",
        TaskStatus::Review => "In Review",
        TaskStatus::Cancelled => "Cancelled",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Task, Priority};
    use sqlx::SqlitePool;
    use uuid::Uuid;
    
    async fn setup_test_service() -> ExportService {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = Arc::new(Repository::new(pool));
        
        // Create test tasks
        for i in 0..3 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {}", i + 1),
                description: format!("Description for task {}", i + 1),
                status: match i {
                    0 => TaskStatus::Todo,
                    1 => TaskStatus::InProgress,
                    _ => TaskStatus::Done,
                },
                priority: match i {
                    0 => Priority::High,
                    1 => Priority::Medium,
                    _ => Priority::Low,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                due_date: if i == 0 { Some(Utc::now() + chrono::Duration::days(7)) } else { None },
                estimated_hours: Some((i + 1) as f32 * 2.0),
                actual_hours: if i == 2 { Some(5.0) } else { None },
                assigned_resource_id: None,
                goal_id: None,
                parent_task_id: None,
                tags: {
                    let mut set = std::collections::HashSet::new();
                    set.insert(format!("tag{}", i));
                    set
                },
                assignee: if i == 0 { Some("Alice".to_string()) } else { None },
                position: crate::domain::task::Position { x: 0.0, y: 0.0 },
            };
            repo.tasks.create(&task).await.unwrap();
        }
        
        ExportService::new(repo)
    }
    
    #[tokio::test]
    async fn test_export_to_json() {
        let service = setup_test_service().await;
        
        let json = service.export_to_json(TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        assert!(json.contains("\"title\": \"Task 1\""));
        assert!(json.contains("\"status\": \"Todo\""));
        assert!(json.contains("\"priority\": \"High\""));
        
        // Verify it's valid JSON
        let parsed: Vec<ExportedTask> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 3);
    }
    
    #[tokio::test]
    async fn test_export_to_csv() {
        let service = setup_test_service().await;
        
        let csv = service.export_to_csv(TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        // Check headers
        assert!(csv.contains("ID,Title,Description,Status,Priority"));
        
        // Check data
        assert!(csv.contains("Task 1"));
        assert!(csv.contains("Todo"));
        assert!(csv.contains("High"));
        assert!(csv.contains("Alice"));
        
        // Verify CSV structure
        let lines: Vec<_> = csv.lines().collect();
        assert!(lines.len() >= 4); // Header + 3 tasks
    }
    
    #[tokio::test]
    async fn test_export_to_markdown() {
        let service = setup_test_service().await;
        
        let markdown = service.export_to_markdown(TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        // Check structure
        assert!(markdown.contains("# Task Export"));
        assert!(markdown.contains("## To Do (1)"));
        assert!(markdown.contains("## In Progress (1)"));
        assert!(markdown.contains("## Done (1)"));
        
        // Check task details
        assert!(markdown.contains("### Task 1"));
        assert!(markdown.contains("- **Priority:** High"));
        assert!(markdown.contains("- **Assignee:** Alice"));
        assert!(markdown.contains("- **Estimated:** 2 hours"));
    }
    
    #[tokio::test]
    async fn test_filtered_export() {
        let service = setup_test_service().await;
        
        // Export only Todo tasks
        let json = service.export_to_json(TaskFilters {
            status: Some(TaskStatus::Todo),
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        let parsed: Vec<ExportedTask> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].title, "Task 1");
    }
}