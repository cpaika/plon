use std::process::Command;
use std::path::PathBuf;
use uuid::Uuid;
use anyhow::Result;
use crate::repository::Repository;
use crate::domain::task_execution::{TaskExecution, ExecutionStatus};

pub struct ClaudeConsole;

impl ClaudeConsole {
    /// Open Claude Code console for an active execution
    pub fn open_console(workspace_dir: &PathBuf, execution: &TaskExecution) -> Result<()> {
        println!("🖥️ Opening Claude console for execution: {}", execution.id);
        
        // Find the Claude command
        let claude_path = Self::find_claude_command()?;
        
        // Open Claude in the workspace directory with the task context
        // Claude will open in interactive mode
        let child = Command::new(&claude_path)
            .current_dir(workspace_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        
        match child {
            Ok(process) => {
                println!("✅ Claude console opened (PID: {:?})", process.id());
                println!("📁 Working directory: {:?}", workspace_dir);
                println!("🌿 Branch: {}", execution.branch_name);
                Ok(())
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to open Claude console: {}", e))
            }
        }
    }
    
    /// Open terminal to show execution logs
    pub fn open_logs_terminal(workspace_dir: &PathBuf, execution: &TaskExecution) -> Result<()> {
        println!("📋 Opening logs for execution: {}", execution.id);
        
        // Create a script to show git log and execution status
        let script = format!(
            r#"#!/bin/bash
echo "📊 Task Execution Monitor"
echo "========================"
echo "Task ID: {}"
echo "Branch: {}"
echo "Status: {:?}"
echo ""
echo "📝 Git Log (last 10 commits):"
echo "----------------------------"
git log --oneline -10
echo ""
echo "📁 Changed Files:"
echo "----------------"
git status --short
echo ""
echo "🔄 Watching for changes... (Press Ctrl+C to exit)"
# Keep terminal open
read -p "Press Enter to close..."
"#,
            execution.task_id,
            execution.branch_name,
            execution.status
        );
        
        let script_path = workspace_dir.join(format!(".monitor_{}.sh", execution.id));
        std::fs::write(&script_path, script)?;
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }
        
        // Open terminal with the script
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-a")
                .arg("Terminal")
                .arg(&script_path)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            // Try common terminal emulators
            let terminals = ["gnome-terminal", "konsole", "xterm", "terminator"];
            let mut opened = false;
            
            for terminal in &terminals {
                if Command::new(terminal)
                    .arg("-e")
                    .arg(&script_path)
                    .spawn()
                    .is_ok()
                {
                    opened = true;
                    break;
                }
            }
            
            if !opened {
                return Err(anyhow::anyhow!("Could not find a terminal emulator"));
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/c", "start", "cmd", "/k"])
                .arg(&script_path)
                .spawn()?;
        }
        
        Ok(())
    }
    
    /// Get real-time status of an execution
    pub async fn get_execution_status(
        repo: &Repository,
        execution_id: Uuid
    ) -> Result<Option<ExecutionStatus>> {
        if let Some(execution) = repo.task_executions.get(execution_id).await? {
            Ok(Some(execution.status))
        } else {
            Ok(None)
        }
    }
    
    /// Check if there's an active execution for a task
    pub async fn get_active_execution(
        repo: &Repository,
        task_id: Uuid
    ) -> Result<Option<TaskExecution>> {
        repo.task_executions.get_active_for_task(task_id).await
    }
    
    fn find_claude_command() -> Result<PathBuf> {
        // Check common locations for Claude CLI
        let possible_paths = vec![
            dirs::home_dir()
                .map(|h| h.join(".claude").join("local").join("claude")),
            Some(PathBuf::from("claude")),
            Some(PathBuf::from("/usr/local/bin/claude")),
        ];
        
        for path_option in possible_paths {
            if let Some(path) = path_option {
                if path.exists() || path == PathBuf::from("claude") {
                    // Try to verify it works
                    let test = Command::new(&path)
                        .arg("--version")
                        .output();
                    
                    if let Ok(output) = test {
                        if output.status.success() {
                            return Ok(path);
                        }
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Claude Code CLI not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task_execution::{TaskExecution, ExecutionStatus};
    use crate::repository::{Repository, database::init_database};
    use uuid::Uuid;
    use tempfile::tempdir;

    async fn setup_test_db() -> (Repository, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
        (Repository::new(pool), dir)
    }

    fn create_test_execution() -> TaskExecution {
        TaskExecution::new(
            Uuid::new_v4(),
            "test-branch".to_string(),
        )
    }

    #[tokio::test]
    async fn test_get_active_execution_none() {
        let (repo, _dir) = setup_test_db().await;
        let task_id = Uuid::new_v4();
        
        let result = ClaudeConsole::get_active_execution(&repo, task_id).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_active_execution_with_running() {
        let (repo, _dir) = setup_test_db().await;
        
        // First create a task
        use crate::domain::task::{Task, TaskStatus, Priority, Position};
        use std::collections::{HashMap, HashSet};
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            status: TaskStatus::InProgress,
            priority: Priority::Medium,
            position: Position { x: 0.0, y: 0.0 },
            metadata: HashMap::new(),
            tags: HashSet::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            scheduled_date: None,
            completed_at: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            subtasks: vec![],
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 0,
        };
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Create and save a running execution
        let mut execution = create_test_execution();
        execution.task_id = task_id;
        execution.status = ExecutionStatus::Running;
        
        repo.task_executions.create(&execution).await.unwrap();
        
        let result = ClaudeConsole::get_active_execution(&repo, task_id).await;
        
        assert!(result.is_ok());
        let exec = result.unwrap();
        assert!(exec.is_some());
        assert_eq!(exec.unwrap().status, ExecutionStatus::Running);
    }

    #[tokio::test]
    async fn test_get_execution_status() {
        let (repo, _dir) = setup_test_db().await;
        
        // First create a task
        use crate::domain::task::{Task, TaskStatus, Priority, Position};
        use std::collections::{HashMap, HashSet};
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            status: TaskStatus::InProgress,
            priority: Priority::Medium,
            position: Position { x: 0.0, y: 0.0 },
            metadata: HashMap::new(),
            tags: HashSet::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            scheduled_date: None,
            completed_at: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            subtasks: vec![],
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 0,
        };
        repo.tasks.create(&task).await.unwrap();
        
        let mut execution = create_test_execution();
        execution.task_id = task.id;
        let exec_id = execution.id;
        
        repo.task_executions.create(&execution).await.unwrap();
        
        let status = ClaudeConsole::get_execution_status(&repo, exec_id).await;
        
        assert!(status.is_ok());
        assert_eq!(status.unwrap(), Some(ExecutionStatus::Running));
    }

    #[test]
    fn test_open_logs_terminal_script_generation() {
        let dir = tempdir().unwrap();
        let workspace_dir = dir.path().to_path_buf();
        let execution = create_test_execution();
        
        // Initialize git repo for the test
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&workspace_dir)
            .output()
            .ok();
        
        // This will create the monitoring script
        let _result = ClaudeConsole::open_logs_terminal(&workspace_dir, &execution);
        
        // Check that the script was created
        let script_path = workspace_dir.join(format!(".monitor_{}.sh", execution.id));
        assert!(script_path.exists());
        
        // Read the script and verify it contains expected content
        let script_content = std::fs::read_to_string(&script_path).unwrap();
        assert!(script_content.contains(&execution.task_id.to_string()));
        assert!(script_content.contains(&execution.branch_name));
        assert!(script_content.contains("Git Log"));
    }
}