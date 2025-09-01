use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeCodeSession {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: SessionStatus,
    pub branch_name: Option<String>,
    pub pr_url: Option<String>,
    pub pr_number: Option<i32>,
    pub session_log: String,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Pending,
    Initializing,
    Working,
    CreatingPR,
    Completed,
    Failed,
    Cancelled,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Initializing => "initializing",
            Self::Working => "working",
            Self::CreatingPR => "creating_pr",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }


    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        !self.is_terminal() && *self != Self::Pending
    }
}

impl FromStr for SessionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "initializing" => Ok(Self::Initializing),
            "working" => Ok(Self::Working),
            "creating_pr" => Ok(Self::CreatingPR),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown session status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub id: Uuid,
    pub github_repo: String,
    pub github_owner: String,
    pub github_token: Option<String>,
    pub claude_api_key: Option<String>,
    pub default_base_branch: String,
    pub auto_create_pr: bool,
    pub working_directory: Option<String>,
    pub claude_model: String,
    pub max_session_duration_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudePromptTemplate {
    pub id: Uuid,
    pub name: String,
    pub template: String,
    pub description: Option<String>,
    pub variables: Vec<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ClaudeCodeSession {
    pub fn new(task_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id,
            status: SessionStatus::Pending,
            branch_name: None,
            pr_url: None,
            pr_number: None,
            session_log: String::new(),
            error_message: None,
            started_at: now,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: SessionStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        if status.is_terminal() {
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn append_log(&mut self, message: &str) {
        self.session_log.push_str(&format!(
            "[{}] {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            message
        ));
        self.updated_at = Utc::now();
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error.clone());
        self.append_log(&format!("ERROR: {}", error));
        self.update_status(SessionStatus::Failed);
    }

    pub fn set_pr_info(&mut self, pr_url: String, pr_number: i32) {
        self.pr_url = Some(pr_url);
        self.pr_number = Some(pr_number);
        self.updated_at = Utc::now();
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }

    pub fn is_timed_out(&self, max_duration_minutes: i32) -> bool {
        let elapsed = Utc::now() - self.started_at;
        elapsed.num_minutes() > max_duration_minutes as i64
    }
}

impl ClaudeCodeConfig {
    pub fn new(github_repo: String, github_owner: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            github_repo,
            github_owner,
            github_token: None,
            claude_api_key: None,
            default_base_branch: "main".to_string(),
            auto_create_pr: true,
            working_directory: None,
            claude_model: "claude-3-opus-20240229".to_string(),
            max_session_duration_minutes: 60,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.github_repo.is_empty() {
            return Err("GitHub repository name is required".to_string());
        }
        if self.github_owner.is_empty() {
            return Err("GitHub owner is required".to_string());
        }
        if self.max_session_duration_minutes < 5 {
            return Err("Session duration must be at least 5 minutes".to_string());
        }
        if self.max_session_duration_minutes > 240 {
            return Err("Session duration cannot exceed 4 hours".to_string());
        }
        Ok(())
    }
}

impl ClaudePromptTemplate {
    pub fn new(name: String, template: String) -> Self {
        let now = Utc::now();
        let variables = Self::extract_variables(&template);
        Self {
            id: Uuid::new_v4(),
            name,
            template,
            description: None,
            variables,
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn extract_variables(template: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut vars = Vec::new();
        for cap in re.captures_iter(template) {
            if let Some(var) = cap.get(1) {
                let var_name = var.as_str().to_string();
                if !vars.contains(&var_name) {
                    vars.push(var_name);
                }
            }
        }
        vars
    }

    pub fn render(&self, context: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        for (key, value) in context.iter() {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_conversion() {
        assert_eq!(
            SessionStatus::from_str("pending"),
            Ok(SessionStatus::Pending)
        );
        assert_eq!(
            SessionStatus::from_str("working"),
            Ok(SessionStatus::Working)
        );
        assert_eq!(
            SessionStatus::from_str("completed"),
            Ok(SessionStatus::Completed)
        );
        assert!(SessionStatus::from_str("invalid").is_err());

        assert_eq!(SessionStatus::Pending.as_str(), "pending");
        assert_eq!(SessionStatus::Working.as_str(), "working");
    }

    #[test]
    fn test_session_status_properties() {
        assert!(!SessionStatus::Pending.is_terminal());
        assert!(!SessionStatus::Working.is_terminal());
        assert!(SessionStatus::Completed.is_terminal());
        assert!(SessionStatus::Failed.is_terminal());

        assert!(!SessionStatus::Pending.is_active());
        assert!(SessionStatus::Working.is_active());
        assert!(!SessionStatus::Completed.is_active());
    }

    #[test]
    fn test_claude_code_session() {
        let task_id = Uuid::new_v4();
        let mut session = ClaudeCodeSession::new(task_id);

        assert_eq!(session.task_id, task_id);
        assert_eq!(session.status, SessionStatus::Pending);
        assert!(session.pr_url.is_none());

        session.update_status(SessionStatus::Working);
        assert_eq!(session.status, SessionStatus::Working);
        assert!(session.completed_at.is_none());

        session.append_log("Starting work");
        assert!(session.session_log.contains("Starting work"));

        session.set_pr_info("https://github.com/owner/repo/pull/123".to_string(), 123);
        assert_eq!(
            session.pr_url,
            Some("https://github.com/owner/repo/pull/123".to_string())
        );
        assert_eq!(session.pr_number, Some(123));

        session.update_status(SessionStatus::Completed);
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_prompt_template() {
        let template = ClaudePromptTemplate::new(
            "test".to_string(),
            "Task: {{task_title}}\nPriority: {{priority}}".to_string(),
        );

        assert_eq!(
            template.variables,
            vec!["task_title".to_string(), "priority".to_string()]
        );

        let mut context = HashMap::new();
        context.insert("task_title".to_string(), "Fix bug".to_string());
        context.insert("priority".to_string(), "High".to_string());

        let rendered = template.render(&context);
        assert_eq!(rendered, "Task: Fix bug\nPriority: High");
    }

    #[test]
    fn test_config_validation() {
        let mut config = ClaudeCodeConfig::new("repo".to_string(), "owner".to_string());
        assert!(config.validate().is_ok());

        config.github_repo = String::new();
        assert!(config.validate().is_err());

        config.github_repo = "repo".to_string();
        config.max_session_duration_minutes = 3;
        assert!(config.validate().is_err());

        config.max_session_duration_minutes = 300;
        assert!(config.validate().is_err());
    }
}
