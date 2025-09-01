use std::process::Command;
use std::path::PathBuf;
use uuid::Uuid;
use crate::domain::task::{Task, TaskStatus};
use anyhow::Result;

pub struct ClaudeAutomation {
    workspace_dir: PathBuf,
}

impl ClaudeAutomation {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }
    
    /// Launch Claude Code to work on a specific task
    pub async fn execute_task(&self, task: &Task, _repo_url: &str) -> Result<()> {
        println!("🤖 Launching Claude Code for task: {}", task.title);
        
        // Create a unique branch name for this task
        let branch_name = format!("task/{}-{}", 
            task.id.to_string().split('-').next().unwrap_or("unknown"),
            sanitize_branch_name(&task.title)
        );
        
        // Create a prompt for Claude Code
        let prompt = format!(
            r#"You are working on the following task:

Task: {}
Description: {}
Status: {:?}
Priority: {:?}

Please complete this task following these steps:
1. Review the existing codebase to understand the context
2. Implement the required changes for this task
3. Write appropriate tests if applicable
4. Ensure all tests pass
5. Create descriptive commits with clear messages
6. When complete, create a pull request with a summary of changes

The task should be implemented following best practices and existing code patterns in the repository."#,
            task.title,
            task.description,
            task.status,
            task.priority
        );
        
        // Save prompt to a file that Claude Code can read
        let prompt_file = self.workspace_dir.join(format!(".claude_task_{}.md", task.id));
        std::fs::write(&prompt_file, &prompt)?;
        
        // Launch Claude Code with the task
        let output = Command::new("claude")
            .current_dir(&self.workspace_dir)
            .args(&[
                "code",
                "--task-file", prompt_file.to_str().unwrap(),
                "--branch", &branch_name,
                "--auto-pr",
                "--pr-title", &format!("Complete task: {}", task.title),
            ])
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ Claude Code launched successfully");
                    println!("Output: {}", String::from_utf8_lossy(&result.stdout));
                } else {
                    eprintln!("❌ Claude Code failed: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to launch Claude Code: {}", e);
                
                // Fallback: Try to open in terminal with a simpler command
                self.launch_claude_terminal_fallback(task, &branch_name).await?;
            }
        }
        
        // Clean up prompt file
        let _ = std::fs::remove_file(prompt_file);
        
        Ok(())
    }
    
    /// Fallback method to launch Claude in terminal if the CLI isn't available
    async fn launch_claude_terminal_fallback(&self, task: &Task, branch_name: &str) -> Result<()> {
        println!("📝 Preparing task for manual Claude Code execution...");
        
        // Create and checkout branch
        Command::new("git")
            .current_dir(&self.workspace_dir)
            .args(&["checkout", "-b", branch_name])
            .output()?;
        
        // Create a TODO file with task details
        let todo_file = self.workspace_dir.join("TODO_CLAUDE.md");
        let content = format!(
            r#"# Task for Claude Code

## Task Details
- **ID**: {}
- **Title**: {}
- **Description**: {}
- **Status**: {:?}
- **Priority**: {:?}

## Instructions
Please complete this task by:
1. Implementing the required functionality
2. Writing tests if applicable
3. Creating clear commits
4. Opening a PR when complete

## Branch
You are currently on branch: {}

## Commands to run when complete:
```bash
git add .
git commit -m "Complete task: {}"
git push -u origin {}
gh pr create --title "Complete task: {}" --body "Automated task completion for: {}"
```
"#,
            task.id,
            task.title,
            task.description,
            task.status,
            task.priority,
            branch_name,
            task.title,
            branch_name,
            task.title,
            task.description
        );
        
        std::fs::write(&todo_file, content)?;
        
        println!("📋 Task details written to TODO_CLAUDE.md");
        println!("🚀 Open this directory in Claude Code to start working on the task");
        
        // Try to open VS Code or default editor
        let _ = Command::new("code")
            .current_dir(&self.workspace_dir)
            .arg(".")
            .arg("TODO_CLAUDE.md")
            .spawn();
        
        Ok(())
    }
    
    /// Check the status of a Claude Code task
    pub async fn check_task_status(&self, task_id: Uuid) -> Result<TaskStatus> {
        // Check if there's an open PR for this task
        let output = Command::new("gh")
            .current_dir(&self.workspace_dir)
            .args(&["pr", "list", "--search", &format!("task/{}", task_id.to_string().split('-').next().unwrap_or("unknown"))])
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                // There's a PR, task is in review
                return Ok(TaskStatus::Review);
            }
        }
        
        // Check if branch exists
        let output = Command::new("git")
            .current_dir(&self.workspace_dir)
            .args(&["branch", "--list", &format!("task/{}*", task_id.to_string().split('-').next().unwrap_or("unknown"))])
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                // Branch exists, task is in progress
                return Ok(TaskStatus::InProgress);
            }
        }
        
        Ok(TaskStatus::Todo)
    }
}

fn sanitize_branch_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}