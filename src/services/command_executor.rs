use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Output, Stdio};
use tokio::process::Command;

/// Trait for executing system commands - allows for mocking in tests
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(
        &self,
        program: &str,
        args: &[&str],
        working_dir: Option<&Path>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<CommandOutput>;
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

impl From<Output> for CommandOutput {
    fn from(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        }
    }
}

/// Real implementation that executes actual system commands
pub struct SystemCommandExecutor;

#[async_trait]
impl CommandExecutor for SystemCommandExecutor {
    async fn execute(
        &self,
        program: &str,
        args: &[&str],
        working_dir: Option<&Path>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<CommandOutput> {
        let mut cmd = Command::new(program);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        if let Some(vars) = env_vars {
            for (key, value) in vars {
                cmd.env(key, value);
            }
        }

        let output = cmd.output().await?;
        Ok(CommandOutput::from(output))
    }
}

pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::time::{Duration, sleep};

    /// Mock implementation for testing
    #[derive(Clone)]
    pub struct MockCommandExecutor {
        responses: Arc<Mutex<Vec<MockResponse>>>,
        call_history: Arc<Mutex<Vec<MockCall>>>,
        default_delay_ms: Arc<Mutex<u64>>,
    }

    #[derive(Debug, Clone)]
    pub struct MockResponse {
        pub program: String,
        pub args_contains: Vec<String>,
        pub stdout: String,
        pub stderr: String,
        pub success: bool,
        pub exit_code: Option<i32>,
        pub delay_ms: Option<u64>,
    }

    #[derive(Debug, Clone)]
    pub struct MockCall {
        pub program: String,
        pub args: Vec<String>,
        pub working_dir: Option<String>,
        pub env_vars: Option<HashMap<String, String>>,
    }

    impl MockCommandExecutor {
        pub fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(Vec::new())),
                call_history: Arc::new(Mutex::new(Vec::new())),
                default_delay_ms: Arc::new(Mutex::new(100)),
            }
        }

        pub fn with_delay(self, delay_ms: u64) -> Self {
            *self.default_delay_ms.lock().unwrap() = delay_ms;
            self
        }

        pub fn add_response(
            &self,
            program: &str,
            args_contains: Vec<&str>,
            stdout: &str,
            stderr: &str,
            success: bool,
        ) {
            self.responses.lock().unwrap().push(MockResponse {
                program: program.to_string(),
                args_contains: args_contains.iter().map(|s| s.to_string()).collect(),
                stdout: stdout.to_string(),
                stderr: stderr.to_string(),
                success,
                exit_code: if success { Some(0) } else { Some(1) },
                delay_ms: None,
            });
        }

        pub fn add_response_with_delay(
            &self,
            program: &str,
            args_contains: Vec<&str>,
            stdout: &str,
            stderr: &str,
            success: bool,
            delay_ms: u64,
        ) {
            self.responses.lock().unwrap().push(MockResponse {
                program: program.to_string(),
                args_contains: args_contains.iter().map(|s| s.to_string()).collect(),
                stdout: stdout.to_string(),
                stderr: stderr.to_string(),
                success,
                exit_code: if success { Some(0) } else { Some(1) },
                delay_ms: Some(delay_ms),
            });
        }

        pub fn get_call_history(&self) -> Vec<MockCall> {
            self.call_history.lock().unwrap().clone()
        }

        pub fn assert_called_with(&self, program: &str, args_contains: &[&str]) -> bool {
            let history = self.call_history.lock().unwrap();
            history.iter().any(|call| {
                call.program == program
                    && args_contains
                        .iter()
                        .all(|arg| call.args.iter().any(|a| a.contains(arg)))
            })
        }

        pub fn mock_claude_success(&self) {
            self.add_response(
                "claude",
                vec!["code"],
                "Starting Claude Code session...\nAnalyzing task requirements...\nImplementing solution...\nRunning tests...\nAll tests passed!\nTask completed successfully",
                "",
                true,
            );
        }

        pub fn mock_claude_error(&self) {
            self.add_response(
                "claude",
                vec!["code"],
                "",
                "Error: Failed to understand task requirements\nPlease provide more detailed instructions",
                false,
            );
        }

        pub fn mock_git_operations(&self) {
            // Git init
            self.add_response(
                "git",
                vec!["init"],
                "Initialized empty Git repository",
                "",
                true,
            );

            // Git config
            self.add_response("git", vec!["config", "user.name"], "", "", true);
            self.add_response("git", vec!["config", "user.email"], "", "", true);

            // Git checkout
            self.add_response(
                "git",
                vec!["checkout", "-b"],
                "Switched to a new branch",
                "",
                true,
            );

            // Git push
            self.add_response("git", vec!["push"], "Branch pushed successfully", "", true);
        }

        pub fn mock_gh_pr_create(&mut self) {
            self.add_response(
                "gh",
                vec!["pr", "create"],
                "https://github.com/test-owner/test-repo/pull/42",
                "Creating pull request...\nPull request created successfully",
                true,
            );
        }

        pub fn mock_gh_auth(&mut self) {
            self.add_response(
                "gh",
                vec!["auth", "status"],
                "github.com\n  ✓ Logged in to github.com as test-user\n  ✓ Git operations for github.com configured to use https protocol.\n  ✓ Token: *******************",
                "",
                true,
            );
        }
    }

    #[async_trait]
    impl CommandExecutor for MockCommandExecutor {
        async fn execute(
            &self,
            program: &str,
            args: &[&str],
            working_dir: Option<&Path>,
            env_vars: Option<HashMap<String, String>>,
        ) -> Result<CommandOutput> {
            // Record the call
            let call = MockCall {
                program: program.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
                working_dir: working_dir.map(|p| p.to_string_lossy().to_string()),
                env_vars: env_vars.clone(),
            };
            self.call_history.lock().unwrap().push(call);

            // Find matching response
            let response = {
                let responses = self.responses.lock().unwrap();
                responses
                    .iter()
                    .find(|r| {
                        r.program == program
                            && r.args_contains
                                .iter()
                                .all(|arg| args.iter().any(|a| a.contains(arg.as_str())))
                    })
                    .cloned()
            };

            if let Some(resp) = response {
                // Simulate delay
                let delay = resp
                    .delay_ms
                    .unwrap_or(*self.default_delay_ms.lock().unwrap());
                if delay > 0 {
                    sleep(Duration::from_millis(delay)).await;
                }

                Ok(CommandOutput {
                    stdout: resp.stdout,
                    stderr: resp.stderr,
                    success: resp.success,
                    exit_code: resp.exit_code,
                })
            } else {
                // Default response if no match
                Ok(CommandOutput {
                    stdout: format!("Mock: {} executed with args: {:?}", program, args),
                    stderr: String::new(),
                    success: true,
                    exit_code: Some(0),
                })
            }
        }
    }

    impl Default for MockCommandExecutor {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockCommandExecutor;
    use super::*;

    #[tokio::test]
    async fn test_mock_command_executor() {
        let executor = MockCommandExecutor::new();

        // Add mock response
        executor.add_response("echo", vec!["hello"], "hello world\n", "", true);

        // Execute command
        let result = executor
            .execute("echo", &["hello", "world"], None, None)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.stdout, "hello world\n");
        assert_eq!(result.stderr, "");

        // Check call history
        assert!(executor.assert_called_with("echo", &["hello"]));
    }

    #[tokio::test]
    async fn test_mock_with_delay() {
        let executor = MockCommandExecutor::new();

        executor.add_response_with_delay(
            "sleep",
            vec!["1"],
            "Slept for 1 second",
            "",
            true,
            200, // 200ms delay
        );

        let start = std::time::Instant::now();
        let result = executor.execute("sleep", &["1"], None, None).await.unwrap();
        let elapsed = start.elapsed();

        assert!(result.success);
        assert!(elapsed.as_millis() >= 200);
    }

    #[tokio::test]
    async fn test_mock_claude_scenarios() {
        let executor = MockCommandExecutor::new();

        // Setup Claude success scenario
        executor.mock_claude_success();

        let result = executor
            .execute("claude", &["code", "--file", "task.md"], None, None)
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("Task completed successfully"));
    }
}
