use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

/// Service for managing workspace directories and files
pub struct WorkspaceService {
    home_dir: String,
}

impl WorkspaceService {
    pub fn new() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        Self { home_dir }
    }
    
    /// Get the path to a workspace directory
    pub fn get_workspace_path(&self, workspace: WorkspaceType) -> PathBuf {
        let path_str = match workspace {
            WorkspaceType::Projects => format!("{}/plon-projects", self.home_dir),
            WorkspaceType::Backups => format!("{}/plon-backups", self.home_dir),
            WorkspaceType::Templates => format!("{}/plon-templates", self.home_dir),
            WorkspaceType::Config => format!("{}/.plon", self.home_dir),
            WorkspaceType::Cache => format!("{}/.plon/cache", self.home_dir),
            WorkspaceType::Logs => format!("{}/.plon/logs", self.home_dir),
        };
        PathBuf::from(path_str)
    }
    
    /// Create all workspace directories if they don't exist
    pub async fn create_all_directories(&self) -> Result<()> {
        let workspaces = vec![
            WorkspaceType::Projects,
            WorkspaceType::Backups,
            WorkspaceType::Templates,
            WorkspaceType::Config,
            WorkspaceType::Cache,
            WorkspaceType::Logs,
        ];
        
        for workspace in workspaces {
            let path = self.get_workspace_path(workspace);
            if !path.exists() {
                fs::create_dir_all(&path).await?;
                println!("✅ Created workspace directory: {}", path.display());
            }
        }
        
        // Create README in projects directory
        let readme_path = self.get_workspace_path(WorkspaceType::Projects).join("README.md");
        if !readme_path.exists() {
            let readme_content = r#"# Plon Workspace

This directory contains your Plon projects and tasks.

## Directory Structure

- `plon-projects/` - Your active projects and tasks
- `plon-backups/` - Automatic backups of your data  
- `plon-templates/` - Reusable task templates
- `.plon/` - Configuration and cache files

## Getting Started

Create new tasks through the Plon desktop or web application.

## Task Organization

Tasks are organized by project and can be:
- Viewed in Map view for spatial organization
- Listed in List view for detailed management
- Organized in Kanban view for workflow tracking
- Scheduled in Timeline and Gantt views

"#;
            fs::write(&readme_path, readme_content).await?;
            println!("📝 Created workspace README");
        }
        
        Ok(())
    }
    
    /// Create a project directory
    pub async fn create_project_directory(&self, project_name: &str) -> Result<PathBuf> {
        let project_path = self.get_workspace_path(WorkspaceType::Projects).join(project_name);
        
        if !project_path.exists() {
            fs::create_dir_all(&project_path).await?;
            
            // Create project structure
            let subdirs = vec!["src", "docs", "tests", "resources"];
            for subdir in subdirs {
                let subdir_path = project_path.join(subdir);
                fs::create_dir_all(&subdir_path).await?;
            }
            
            // Create project README
            let readme_path = project_path.join("README.md");
            let readme_content = format!(r#"# {}

## Description

Project created by Plon task management system.

## Structure

- `src/` - Source code and implementation
- `docs/` - Documentation
- `tests/` - Test files
- `resources/` - Additional resources

## Tasks

View and manage tasks for this project in the Plon application.

"#, project_name);
            fs::write(&readme_path, readme_content).await?;
        }
        
        Ok(project_path)
    }
    
    /// Get or create a task directory
    pub async fn get_or_create_task_directory(&self, task_id: &str, task_title: &str) -> Result<PathBuf> {
        // Sanitize task title for filesystem
        let safe_title = sanitize_filename(task_title);
        let task_dir_name = format!("{}_{}", task_id.chars().take(8).collect::<String>(), safe_title);
        
        let task_path = self.get_workspace_path(WorkspaceType::Projects).join(task_dir_name);
        
        if !task_path.exists() {
            fs::create_dir_all(&task_path).await?;
            
            // Create initial task file
            let task_file = task_path.join("TODO_CLAUDE.md");
            let task_content = format!(r#"# Task: {}

## Task ID: {}

## Instructions

This task has been prepared for Claude Code. 

1. Review the task requirements below
2. Implement the solution
3. Test your implementation
4. Update task status when complete

## Task Details

{}

## Acceptance Criteria

- [ ] Implementation complete
- [ ] Tests passing
- [ ] Documentation updated

## Notes

Add any implementation notes here.

"#, task_title, task_id, task_title);
            
            fs::write(&task_file, task_content).await?;
        }
        
        Ok(task_path)
    }
    
    /// Clean up old backup files
    pub async fn cleanup_old_backups(&self, keep_count: usize) -> Result<()> {
        let backup_path = self.get_workspace_path(WorkspaceType::Backups);
        
        if backup_path.exists() {
            let mut entries = fs::read_dir(&backup_path).await?;
            let mut backups = Vec::new();
            
            while let Some(entry) = entries.next_entry().await? {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("db") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            backups.push((entry.path(), modified));
                        }
                    }
                }
            }
            
            // Sort by modification time (newest first)
            backups.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Remove old backups
            for (path, _) in backups.iter().skip(keep_count) {
                fs::remove_file(path).await?;
                println!("🗑️ Removed old backup: {}", path.display());
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WorkspaceType {
    Projects,
    Backups,
    Templates,
    Config,
    Cache,
    Logs,
}

/// Sanitize a filename to be filesystem-safe
pub(crate) fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .take(50) // Limit length
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file:name"), "test_file_name");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
        assert_eq!(sanitize_filename("test<>file|name"), "test__file_name");
    }
    
    #[tokio::test]
    async fn test_workspace_paths() {
        let service = WorkspaceService::new();
        
        let projects_path = service.get_workspace_path(WorkspaceType::Projects);
        assert!(projects_path.to_string_lossy().contains("plon-projects"));
        
        let config_path = service.get_workspace_path(WorkspaceType::Config);
        assert!(config_path.to_string_lossy().contains(".plon"));
    }
}