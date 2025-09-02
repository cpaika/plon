#[cfg(test)]
mod tests {
    use super::super::workspace_service::*;
    use tempfile::TempDir;
    use std::env;
    
    #[tokio::test]
    async fn test_create_workspace_directories() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        
        // Override HOME for testing
        env::set_var("HOME", temp_path);
        
        let service = WorkspaceService::new();
        
        // Create all directories
        service.create_all_directories().await.unwrap();
        
        // Verify all directories exist
        assert!(service.get_workspace_path(WorkspaceType::Projects).exists());
        assert!(service.get_workspace_path(WorkspaceType::Backups).exists());
        assert!(service.get_workspace_path(WorkspaceType::Templates).exists());
        assert!(service.get_workspace_path(WorkspaceType::Config).exists());
        assert!(service.get_workspace_path(WorkspaceType::Cache).exists());
        assert!(service.get_workspace_path(WorkspaceType::Logs).exists());
        
        // Verify README was created
        let readme_path = service.get_workspace_path(WorkspaceType::Projects).join("README.md");
        assert!(readme_path.exists());
        
        // Verify README content
        let readme_content = tokio::fs::read_to_string(&readme_path).await.unwrap();
        assert!(readme_content.contains("Plon Workspace"));
    }
    
    #[tokio::test]
    async fn test_create_project_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("HOME", temp_path);
        
        let service = WorkspaceService::new();
        service.create_all_directories().await.unwrap();
        
        // Create a project directory
        let project_path = service.create_project_directory("test-project").await.unwrap();
        
        // Verify project structure
        assert!(project_path.exists());
        assert!(project_path.join("src").exists());
        assert!(project_path.join("docs").exists());
        assert!(project_path.join("tests").exists());
        assert!(project_path.join("resources").exists());
        assert!(project_path.join("README.md").exists());
        
        // Verify README content
        let readme_content = tokio::fs::read_to_string(project_path.join("README.md")).await.unwrap();
        assert!(readme_content.contains("test-project"));
    }
    
    #[tokio::test]
    async fn test_get_or_create_task_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("HOME", temp_path);
        
        let service = WorkspaceService::new();
        service.create_all_directories().await.unwrap();
        
        // Create a task directory
        let task_id = "12345678-1234-1234-1234-123456789012";
        let task_title = "Test Task: With Special/Characters*";
        let task_path = service.get_or_create_task_directory(task_id, task_title).await.unwrap();
        
        // Verify task directory exists
        assert!(task_path.exists());
        
        // Verify directory name is sanitized
        let dir_name = task_path.file_name().unwrap().to_str().unwrap();
        assert!(dir_name.starts_with("12345678_"));
        assert!(!dir_name.contains("/"));
        assert!(!dir_name.contains("*"));
        
        // Verify TODO_CLAUDE.md was created
        let todo_file = task_path.join("TODO_CLAUDE.md");
        assert!(todo_file.exists());
        
        let todo_content = tokio::fs::read_to_string(&todo_file).await.unwrap();
        assert!(todo_content.contains(task_id));
    }
    
    #[tokio::test]
    async fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("HOME", temp_path);
        
        let service = WorkspaceService::new();
        service.create_all_directories().await.unwrap();
        
        let backup_path = service.get_workspace_path(WorkspaceType::Backups);
        
        // Create some test backup files
        for i in 1..=5 {
            let file_path = backup_path.join(format!("backup_{}.db", i));
            tokio::fs::write(&file_path, format!("backup {}", i)).await.unwrap();
            // Add a small delay to ensure different modification times
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // Keep only 3 backups
        service.cleanup_old_backups(3).await.unwrap();
        
        // Count remaining backup files
        let mut entries = tokio::fs::read_dir(&backup_path).await.unwrap();
        let mut count = 0;
        while let Some(_entry) = entries.next_entry().await.unwrap() {
            count += 1;
        }
        
        assert_eq!(count, 3, "Should keep only 3 backup files");
    }
    
    #[test]
    fn test_sanitize_filename() {
        use super::super::workspace_service::sanitize_filename;
        
        assert_eq!(sanitize_filename("normal_file"), "normal_file");
        assert_eq!(sanitize_filename("file/with/slashes"), "file_with_slashes");
        assert_eq!(sanitize_filename("file:with:colons"), "file_with_colons");
        assert_eq!(sanitize_filename("file*with?special<>chars|"), "file_with_special__chars_");
        assert_eq!(sanitize_filename("file\"with\"quotes"), "file_with_quotes");
        
        // Test length limiting
        let long_name = "a".repeat(100);
        assert_eq!(sanitize_filename(&long_name).len(), 50);
    }
}