use anyhow::Result;
use plon::domain::claude_code::{
    ClaudeCodeConfig, ClaudeCodeSession, ClaudePromptTemplate, SessionStatus,
};
use plon::domain::task::Task;
use plon::repository::Repository;
use plon::services::ClaudeCodeService;
use plon::services::command_executor::{CommandExecutor, mock::MockCommandExecutor};
use sqlx::SqlitePool;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

pub struct ClaudeCodeTestEnvironment {
    pub repository: Repository,
    pub service: ClaudeCodeService,
    pub temp_dir: TempDir,
    pub config: ClaudeCodeConfig,
    pub template: ClaudePromptTemplate,
    pub mock_executor: MockCommandExecutor,
}

impl ClaudeCodeTestEnvironment {
    pub async fn new() -> Result<Self> {
        // Create temp directory for test
        let temp_dir = TempDir::new()?;

        // Set up test database
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        let repository = Repository::new(pool);

        // Create test configuration
        let mut config = ClaudeCodeConfig::new("test-repo".to_string(), "test-owner".to_string());
        config.working_directory = Some(temp_dir.path().to_string_lossy().to_string());
        config.auto_create_pr = true;
        config.max_session_duration_minutes = 5;

        // Save config to database
        repository.claude_code.create_config(&config).await?;

        // Create default template
        let template = ClaudePromptTemplate::new(
            "test-template".to_string(),
            "Test task: {{task_title}}\nDescription: {{task_description}}".to_string(),
        );
        repository.claude_code.create_template(&template).await?;

        // Create mock executor
        let mut mock_executor = MockCommandExecutor::new();
        mock_executor.mock_git_operations();

        // Create service with mock executor
        let service = ClaudeCodeService::with_executor(
            repository.claude_code.clone(),
            Arc::new(mock_executor.clone()),
        );

        // Initialize git repo in temp dir
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(temp_dir.path())
            .output()?;

        std::process::Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()?;

        std::process::Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()?;

        Ok(Self {
            repository,
            service,
            temp_dir,
            config,
            template,
            mock_executor,
        })
    }

    pub fn create_test_task(&self, title: &str, description: &str) -> Task {
        let mut task = Task::new(title.to_string(), description.to_string());
        task.estimated_hours = Some(2.0);
        task.add_tag("test".to_string());
        task
    }

    pub async fn launch_with_mock(&mut self, task: &Task, mode: &str) -> Result<ClaudeCodeSession> {
        // Configure mock based on mode
        // Clone the mock to get a mutable version
        let mut mock = self.mock_executor.clone();

        match mode {
            "success" => {
                mock.mock_claude_success();
                mock.mock_gh_pr_create();
            }
            "error" => {
                mock.mock_claude_error();
            }
            "partial" => {
                mock.add_response(
                    "claude",
                    vec!["code"],
                    "Starting Claude Code session...\nAnalyzing task requirements...\nImplementing solution...\nWarning: Some tests are failing\nPartial implementation complete",
                    "",
                    true,
                );
            }
            "timeout" => {
                mock.add_response_with_delay(
                    "claude",
                    vec!["code"],
                    "Starting Claude Code session...",
                    "",
                    true,
                    10000, // 10 second delay to simulate timeout
                );
            }
            _ => return Err(anyhow::anyhow!("Unknown mock mode: {}", mode)),
        }

        // Update the service with the new mock
        self.service =
            ClaudeCodeService::with_executor(self.repository.claude_code.clone(), Arc::new(mock));

        // Launch Claude Code
        let session = self
            .service
            .launch_claude_code(task, &self.config, &self.template)
            .await?;

        Ok(session)
    }

    pub async fn wait_for_completion(
        &self,
        session_id: Uuid,
        timeout_secs: u64,
    ) -> Result<ClaudeCodeSession> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed().as_secs() > timeout_secs {
                return Err(anyhow::anyhow!("Timeout waiting for session completion"));
            }

            let session = self
                .repository
                .claude_code
                .get_session(session_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

            if session.status.is_terminal() {
                return Ok(session);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn get_session_logs(&self, session_id: Uuid) -> Result<String> {
        let session = self
            .repository
            .claude_code
            .get_session(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        Ok(session.session_log)
    }

    pub async fn verify_pr_created(&self, session_id: Uuid) -> Result<bool> {
        let session = self
            .repository
            .claude_code
            .get_session(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        Ok(session.pr_url.is_some() && session.pr_number.is_some())
    }

    pub async fn cleanup(self) -> Result<()> {
        // Cleanup is handled automatically when TempDir is dropped
        Ok(())
    }
}

pub struct MockClaudeCodeService {
    pub sessions: Vec<ClaudeCodeSession>,
    pub should_fail: bool,
    pub failure_message: String,
    pub execution_delay_ms: u64,
}

impl MockClaudeCodeService {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            should_fail: false,
            failure_message: String::new(),
            execution_delay_ms: 100,
        }
    }

    pub fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.failure_message = message.to_string();
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.execution_delay_ms = delay_ms;
        self
    }

    pub async fn mock_launch(&mut self, task: &Task) -> Result<ClaudeCodeSession> {
        // Simulate delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.execution_delay_ms)).await;

        if self.should_fail {
            return Err(anyhow::anyhow!(self.failure_message.clone()));
        }

        let mut session = ClaudeCodeSession::new(task.id);
        session.update_status(SessionStatus::Working);
        session.append_log("Mock Claude Code session started");
        session.branch_name = Some(format!("claude/mock-{}", task.id));

        self.sessions.push(session.clone());

        Ok(session)
    }

    pub async fn mock_complete_with_pr(&mut self, session_id: Uuid) -> Result<()> {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            session.update_status(SessionStatus::Completed);
            session.set_pr_info(
                format!(
                    "https://github.com/test/repo/pull/{}",
                    rand::random::<u32>() % 1000
                ),
                rand::random::<i32>() % 1000,
            );
            session.append_log("Mock PR created successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    pub async fn mock_fail(&mut self, session_id: Uuid, error: &str) -> Result<()> {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            session.set_error(error.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
}

// Test fixtures
pub mod fixtures {
    use super::*;

    pub fn sample_task() -> Task {
        let mut task = Task::new(
            "Implement user authentication".to_string(),
            "Add JWT-based authentication with login/logout endpoints".to_string(),
        );
        task.estimated_hours = Some(4.0);
        task.add_tag("backend".to_string());
        task.add_tag("security".to_string());
        task
    }

    pub fn complex_task() -> Task {
        let mut task = Task::new(
            "Refactor database layer".to_string(),
            r#"# Database Refactoring Task

## Objectives
- Migrate from raw SQL to ORM
- Implement connection pooling
- Add query optimization
- Set up migrations

## Technical Requirements
- Use SQLx for async operations
- Implement retry logic
- Add comprehensive logging
- Write unit tests for all queries

## Acceptance Criteria
- [ ] All existing queries migrated
- [ ] Performance improved by 20%
- [ ] 90% test coverage
- [ ] Documentation updated"#
                .to_string(),
        );
        task.estimated_hours = Some(16.0);
        task.add_tag("database".to_string());
        task.add_tag("performance".to_string());
        task.add_tag("refactoring".to_string());
        task
    }

    pub fn minimal_task() -> Task {
        Task::new("Fix typo".to_string(), "Fix typo in README".to_string())
    }

    pub fn sample_config() -> ClaudeCodeConfig {
        let mut config = ClaudeCodeConfig::new("plon".to_string(), "test-user".to_string());
        config.default_base_branch = "main".to_string();
        config.auto_create_pr = true;
        config.max_session_duration_minutes = 30;
        config
    }

    pub fn sample_template() -> ClaudePromptTemplate {
        ClaudePromptTemplate::new(
            "standard".to_string(),
            r#"You are working on the following task:

Title: {{task_title}}
Description: {{task_description}}
Estimated Hours: {{estimated_hours}}
Tags: {{tags}}

Please implement this task following best practices:
1. Write clean, maintainable code
2. Include appropriate tests
3. Update documentation
4. Follow existing code style

When complete, create a pull request with:
- Clear description of changes
- Test results
- Any breaking changes noted"#
                .to_string(),
        )
    }
}
