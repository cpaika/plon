use crate::domain::claude_code::{
    ClaudeCodeConfig, ClaudeCodeSession, ClaudePromptTemplate, SessionStatus,
};
use crate::domain::task::Task;
use crate::repository::claude_code_repository::ClaudeCodeRepository;
use crate::services::command_executor::{CommandExecutor, SystemCommandExecutor};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct ClaudeCodeService {
    repository: ClaudeCodeRepository,
    active_sessions: HashMap<Uuid, JoinHandle<()>>,
    log_sender: mpsc::Sender<(Uuid, String)>,
    log_receiver: Option<mpsc::Receiver<(Uuid, String)>>,
    command_executor: Arc<dyn CommandExecutor>,
}

impl ClaudeCodeService {
    pub fn new(repository: ClaudeCodeRepository) -> Self {
        Self::with_executor(repository, Arc::new(SystemCommandExecutor))
    }

    pub fn with_executor(
        repository: ClaudeCodeRepository,
        executor: Arc<dyn CommandExecutor>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            repository,
            active_sessions: HashMap::new(),
            log_sender: tx,
            log_receiver: Some(rx),
            command_executor: executor,
        }
    }

    pub async fn launch_claude_code(
        &mut self,
        task: &Task,
        config: &ClaudeCodeConfig,
        template: &ClaudePromptTemplate,
    ) -> Result<ClaudeCodeSession> {
        // Validate configuration
        config.validate().map_err(|e| anyhow::anyhow!(e))?;

        // Create new session
        let mut session = ClaudeCodeSession::new(task.id);
        session.append_log("Initializing Claude Code session");
        session.update_status(SessionStatus::Initializing);

        // Save initial session to database
        self.repository.create_session(&session).await?;

        // Prepare working directory
        let work_dir = self.prepare_working_directory(task, config)?;
        session.append_log(&format!("Working directory: {}", work_dir.display()));

        // Generate branch name
        let branch_name = self.generate_branch_name(task);
        session.branch_name = Some(branch_name.clone());
        session.append_log(&format!("Branch name: {}", branch_name));

        // Render prompt from template
        let prompt = self.render_prompt(task, template)?;
        session.append_log("Prompt generated from template");

        // Create prompt file
        let prompt_file = work_dir.join("claude_task.md");
        fs::write(&prompt_file, &prompt)?;

        // Create instructions file for Claude Code
        let instructions = self.create_claude_instructions(task, config, &branch_name)?;
        let instructions_file = work_dir.join("claude_instructions.md");
        fs::write(&instructions_file, &instructions)?;

        // Update session status
        session.update_status(SessionStatus::Working);
        self.repository.update_session(&session).await?;

        // Launch Claude Code process
        let session_id = session.id;
        let log_sender = self.log_sender.clone();
        let repo_clone = self.repository.clone();
        let config_clone = config.clone();
        let executor_clone = self.command_executor.clone();

        let handle = tokio::spawn(async move {
            let result = Self::run_claude_code_process(
                session_id,
                work_dir,
                prompt_file,
                instructions_file,
                branch_name,
                config_clone,
                log_sender.clone(),
                repo_clone,
                executor_clone,
            )
            .await;

            if let Err(e) = result {
                let _ = log_sender
                    .send((session_id, format!("Process error: {}", e)))
                    .await;
            }
        });

        self.active_sessions.insert(session.id, handle);

        Ok(session)
    }

    async fn run_claude_code_process(
        session_id: Uuid,
        work_dir: PathBuf,
        prompt_file: PathBuf,
        instructions_file: PathBuf,
        branch_name: String,
        config: ClaudeCodeConfig,
        log_sender: mpsc::Sender<(Uuid, String)>,
        repository: ClaudeCodeRepository,
        executor: Arc<dyn CommandExecutor>,
    ) -> Result<()> {
        // Setup git branch
        let _ = log_sender
            .send((session_id, "Setting up git branch".to_string()))
            .await;

        executor
            .execute(
                "git",
                &["checkout", "-b", &branch_name],
                Some(&work_dir),
                None,
            )
            .await
            .context("Failed to create git branch")?;

        // Build Claude Code command
        let mut env_vars = HashMap::new();
        if let Some(api_key) = &config.claude_api_key {
            env_vars.insert("ANTHROPIC_API_KEY".to_string(), api_key.clone());
        }

        // Execute Claude Code
        let _ = log_sender
            .send((session_id, "Launching Claude Code".to_string()))
            .await;
        let output = executor
            .execute(
                "claude",
                &[
                    "code",
                    "--file",
                    &prompt_file.to_string_lossy(),
                    "--instructions",
                    &instructions_file.to_string_lossy(),
                ],
                Some(&work_dir),
                if env_vars.is_empty() {
                    None
                } else {
                    Some(env_vars)
                },
            )
            .await
            .context("Failed to execute Claude Code")?;

        // Process output
        if !output.stdout.is_empty() {
            let _ = log_sender
                .send((session_id, format!("Output: {}", output.stdout)))
                .await;
        }

        if !output.stderr.is_empty() {
            let _ = log_sender
                .send((session_id, format!("Error: {}", output.stderr)))
                .await;
        }

        // Check if successful
        if !output.success {
            let mut session = repository
                .get_session(session_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.set_error(format!("Claude Code failed: {}", output.stderr));
            repository.update_session(&session).await?;
            return Ok(());
        }

        // Create PR if configured
        if config.auto_create_pr {
            let _ = log_sender
                .send((session_id, "Creating pull request".to_string()))
                .await;

            let mut session = repository
                .get_session(session_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.update_status(SessionStatus::CreatingPR);
            repository.update_session(&session).await?;

            // Push branch
            executor
                .execute(
                    "git",
                    &["push", "-u", "origin", &branch_name],
                    Some(&work_dir),
                    None,
                )
                .await
                .context("Failed to push branch")?;

            // Create PR using gh CLI
            let pr_output = executor
                .execute(
                    "gh",
                    &[
                        "pr",
                        "create",
                        "--title",
                        &format!("Claude Code: {}", branch_name),
                        "--body",
                        &format!(
                            "Automated PR created by Claude Code for session {}",
                            session_id
                        ),
                        "--base",
                        &config.default_base_branch,
                        "--head",
                        &branch_name,
                    ],
                    Some(&work_dir),
                    None,
                )
                .await
                .context("Failed to create PR")?;

            if pr_output.success {
                let pr_url = pr_output.stdout.trim().to_string();

                // Extract PR number from URL
                let pr_number = pr_url
                    .split('/')
                    .next_back()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);

                let mut session = repository
                    .get_session(session_id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
                session.set_pr_info(pr_url.clone(), pr_number);
                session.update_status(SessionStatus::Completed);
                session.append_log(&format!("PR created: {}", pr_url));
                repository.update_session(&session).await?;

                let _ = log_sender
                    .send((session_id, format!("PR created: {}", pr_url)))
                    .await;
            } else {
                let error = &pr_output.stderr;
                let _ = log_sender
                    .send((session_id, format!("Failed to create PR: {}", error)))
                    .await;

                let mut session = repository
                    .get_session(session_id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
                session.set_error(format!("Failed to create PR: {}", error));
                repository.update_session(&session).await?;
            }
        } else {
            // Mark as completed without PR
            let mut session = repository
                .get_session(session_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.update_status(SessionStatus::Completed);
            session.append_log("Completed without creating PR");
            repository.update_session(&session).await?;
        }

        Ok(())
    }

    pub async fn cancel_session(&mut self, session_id: Uuid) -> Result<()> {
        // Cancel the running task if it exists
        if let Some(handle) = self.active_sessions.remove(&session_id) {
            handle.abort();
        }

        // Update session status
        if let Some(mut session) = self.repository.get_session(session_id).await? {
            session.update_status(SessionStatus::Cancelled);
            session.append_log("Session cancelled by user");
            self.repository.update_session(&session).await?;
        }

        Ok(())
    }

    pub async fn get_session_status(&self, session_id: Uuid) -> Result<Option<ClaudeCodeSession>> {
        self.repository.get_session(session_id).await
    }

    pub async fn get_task_sessions(&self, task_id: Uuid) -> Result<Vec<ClaudeCodeSession>> {
        self.repository.get_sessions_by_task(task_id).await
    }

    pub async fn process_log_updates(&mut self) -> Result<()> {
        if let Some(mut receiver) = self.log_receiver.take() {
            while let Ok((session_id, log_message)) = receiver.try_recv() {
                if let Some(mut session) = self.repository.get_session(session_id).await? {
                    session.append_log(&log_message);
                    self.repository.update_session(&session).await?;
                }
            }
            self.log_receiver = Some(receiver);
        }
        Ok(())
    }

    fn prepare_working_directory(&self, _task: &Task, config: &ClaudeCodeConfig) -> Result<PathBuf> {
        let base_dir = if let Some(dir) = &config.working_directory {
            PathBuf::from(dir)
        } else {
            std::env::current_dir()?
        };

        if !base_dir.exists() {
            return Err(anyhow::anyhow!(
                "Working directory does not exist: {:?}",
                base_dir
            ));
        }

        Ok(base_dir)
    }

    fn generate_branch_name(&self, task: &Task) -> String {
        let task_id_short = task.id.to_string().chars().take(8).collect::<String>();
        let title_slug = task
            .title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string();

        format!("claude/{}-{}", task_id_short, title_slug)
    }

    fn render_prompt(&self, task: &Task, template: &ClaudePromptTemplate) -> Result<String> {
        let mut context = HashMap::new();

        // Basic task information
        context.insert("task_title".to_string(), task.title.clone());
        context.insert("task_description".to_string(), task.description.clone());
        context.insert("task_id".to_string(), task.id.to_string());
        context.insert(
            "task_id_short".to_string(),
            task.id.to_string().chars().take(8).collect(),
        );

        // Task metadata
        context.insert("priority".to_string(), format!("{:?}", task.priority));
        context.insert("status".to_string(), format!("{:?}", task.status));
        context.insert(
            "tags".to_string(),
            task.tags.iter().cloned().collect::<Vec<_>>().join(", "),
        );

        // Optional fields
        if let Some(hours) = task.estimated_hours {
            context.insert("estimated_hours".to_string(), hours.to_string());
        } else {
            context.insert("estimated_hours".to_string(), "Not estimated".to_string());
        }

        // Slugified title for branch naming
        let title_slug = task
            .title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        context.insert("task_title_slug".to_string(), title_slug);

        // Goal information (would need to be passed in or fetched)
        context.insert("goal_title".to_string(), "N/A".to_string());

        Ok(template.render(&context))
    }

    fn create_claude_instructions(
        &self,
        _task: &Task,
        config: &ClaudeCodeConfig,
        branch_name: &str,
    ) -> Result<String> {
        let instructions = format!(
            r#"# Claude Code Instructions

## Task Context
You are working on a task from the Plon project management system.

## Git Configuration
- Repository: {}/{}
- Branch: {}
- Base Branch: {}

## Task Requirements
1. Read and understand the task requirements in claude_task.md
2. Implement the necessary changes
3. Write appropriate tests
4. Ensure code quality and documentation
5. Commit your changes with clear messages

## Pull Request
{}

## Important Notes
- Follow the existing code style and conventions
- Use meaningful commit messages
- Test your changes thoroughly
- Document any significant design decisions

## Completion
When you're done:
1. Ensure all changes are committed
2. The system will automatically create a PR if configured
3. Include a summary of changes in your final output
"#,
            config.github_owner,
            config.github_repo,
            branch_name,
            config.default_base_branch,
            if config.auto_create_pr {
                "A pull request will be automatically created when you complete the task."
            } else {
                "No automatic PR will be created. Manual review and PR creation required."
            }
        );

        Ok(instructions)
    }

    pub async fn cleanup_old_sessions(&self, max_age_days: i32) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days as i64);
        self.repository.cleanup_old_sessions(cutoff).await
    }
}

#[cfg(test)]
#[path = "claude_code_service_tests.rs"]
mod claude_code_service_tests;
