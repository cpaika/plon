use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::repository::Repository;
use crate::services::command_executor::CommandExecutor;

#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub pr_url: String,
    pub pr_number: i32,
    pub title: String,
    pub description: String,
    pub branch: String,
    pub base_branch: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub status: PRStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PRStatus {
    Open,
    Reviewing,
    Approved,
    ChangesRequested,
    Merged,
    Closed,
}

#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub approved: bool,
    pub comments: Vec<String>,
    pub tests_passed: bool,
    pub merge_conflicts: bool,
    pub review_time: DateTime<Utc>,
}

pub struct PRReviewService {
    #[allow(dead_code)]
    repository: Arc<Repository>,
    command_executor: Arc<dyn CommandExecutor>,
}

impl PRReviewService {
    pub fn new(repository: Arc<Repository>, command_executor: Arc<dyn CommandExecutor>) -> Self {
        Self {
            repository,
            command_executor,
        }
    }

    pub async fn review_pr(&self, pr_url: String) -> Result<ReviewResult> {
        // Extract PR info from URL
        let pr_info = self.get_pr_info(&pr_url).await?;

        // Run tests
        let tests_passed = self.run_pr_tests(&pr_info).await?;

        // Check for merge conflicts
        let merge_conflicts = self.check_merge_conflicts(&pr_info).await?;

        // Generate review comments
        let comments = self
            .generate_review_comments(&pr_info, tests_passed, merge_conflicts)
            .await?;

        // Determine approval
        let approved = tests_passed && !merge_conflicts;

        Ok(ReviewResult {
            approved,
            comments,
            tests_passed,
            merge_conflicts,
            review_time: Utc::now(),
        })
    }

    pub async fn approve_and_merge(&self, pr_url: String) -> Result<()> {
        // Review the PR first
        let review_result = self.review_pr(pr_url.clone()).await?;

        if !review_result.approved {
            return Err(anyhow::anyhow!(
                "PR cannot be merged: tests failed or merge conflicts exist"
            ));
        }

        // Approve the PR
        self.approve_pr(&pr_url).await?;

        // Merge the PR
        self.merge_pr(&pr_url).await?;

        Ok(())
    }

    async fn get_pr_info(&self, pr_url: &str) -> Result<PullRequestInfo> {
        // Parse PR URL to extract owner/repo/number
        let parts: Vec<&str> = pr_url.split('/').collect();
        if parts.len() < 7 {
            return Err(anyhow::anyhow!("Invalid PR URL"));
        }

        let owner = parts[3];
        let repo = parts[4];
        let pr_number = parts[6].parse::<i32>()?;

        // Use gh CLI to get PR info
        let pr_number_str = pr_number.to_string();
        let repo_str = format!("{}/{}", owner, repo);
        let args = vec![
            "pr",
            "view",
            &pr_number_str,
            "--repo",
            &repo_str,
            "--json",
            "title,body,headRefName,baseRefName,author,createdAt,state",
        ];

        let output = self
            .command_executor
            .execute("gh", &args, None, None)
            .await?;

        // Parse JSON output
        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;

        Ok(PullRequestInfo {
            pr_url: pr_url.to_string(),
            pr_number,
            title: json["title"].as_str().unwrap_or("").to_string(),
            description: json["body"].as_str().unwrap_or("").to_string(),
            branch: json["headRefName"].as_str().unwrap_or("").to_string(),
            base_branch: json["baseRefName"].as_str().unwrap_or("main").to_string(),
            author: json["author"]["login"].as_str().unwrap_or("").to_string(),
            created_at: Utc::now(), // Simplified for now
            status: match json["state"].as_str().unwrap_or("OPEN") {
                "MERGED" => PRStatus::Merged,
                "CLOSED" => PRStatus::Closed,
                _ => PRStatus::Open,
            },
        })
    }

    async fn run_pr_tests(&self, pr_info: &PullRequestInfo) -> Result<bool> {
        // Check out the PR branch
        let checkout_args = vec!["checkout", &pr_info.branch];
        self.command_executor
            .execute("git", &checkout_args, None, None)
            .await?;

        // Run tests
        let test_result = self
            .command_executor
            .execute("cargo", &["test"], None, None)
            .await;

        // Return to previous branch
        self.command_executor
            .execute("git", &["checkout", "-"], None, None)
            .await?;

        // Check if tests passed based on command success
        match test_result {
            Ok(output) => Ok(output.success),
            Err(_) => Ok(false),
        }
    }

    async fn check_merge_conflicts(&self, pr_info: &PullRequestInfo) -> Result<bool> {
        // Try to merge the branch locally to check for conflicts
        let merge_args = vec!["merge", "--no-commit", "--no-ff", &pr_info.branch];
        let result = self
            .command_executor
            .execute("git", &merge_args, None, None)
            .await;

        // Abort the merge
        let _ = self
            .command_executor
            .execute("git", &["merge", "--abort"], None, None)
            .await;

        // Check if merge failed (indicating conflicts)
        match result {
            Ok(output) => Ok(!output.success), // If merge fails, there are conflicts
            Err(_) => Ok(true), // If command errors, assume conflicts
        }
    }

    async fn generate_review_comments(
        &self,
        pr_info: &PullRequestInfo,
        tests_passed: bool,
        merge_conflicts: bool,
    ) -> Result<Vec<String>> {
        let mut comments = Vec::new();

        comments.push(format!("🤖 Automated PR Review for #{}", pr_info.pr_number));

        if tests_passed {
            comments.push("✅ All tests passed".to_string());
        } else {
            comments.push("❌ Tests failed - please fix before merging".to_string());
        }

        if merge_conflicts {
            comments.push("⚠️ Merge conflicts detected - please resolve".to_string());
        } else {
            comments.push("✅ No merge conflicts".to_string());
        }

        if tests_passed && !merge_conflicts {
            comments.push("🎉 This PR is ready to merge!".to_string());
        }

        Ok(comments)
    }

    async fn approve_pr(&self, pr_url: &str) -> Result<()> {
        // Parse PR URL
        let parts: Vec<&str> = pr_url.split('/').collect();
        let owner = parts[3];
        let repo = parts[4];
        let pr_number = parts[6];

        // Approve using gh CLI
        let repo_str = format!("{}/{}", owner, repo);
        let args = vec![
            "pr",
            "review",
            pr_number,
            "--repo",
            &repo_str,
            "--approve",
            "--body",
            "Automated approval by Claude PR reviewer",
        ];

        self.command_executor
            .execute("gh", &args, None, None)
            .await?;
        Ok(())
    }

    async fn merge_pr(&self, pr_url: &str) -> Result<()> {
        // Parse PR URL
        let parts: Vec<&str> = pr_url.split('/').collect();
        let owner = parts[3];
        let repo = parts[4];
        let pr_number = parts[6];

        // Merge using gh CLI
        let repo_str = format!("{}/{}", owner, repo);
        let args = vec![
            "pr", "merge", pr_number, "--repo", &repo_str, "--squash", "--auto",
        ];

        self.command_executor
            .execute("gh", &args, None, None)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::database::init_test_database;
    use crate::services::command_executor::mock::MockCommandExecutor;

    async fn setup() -> (PRReviewService, Arc<MockCommandExecutor>) {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mock_executor = Arc::new(MockCommandExecutor::new());

        let service = PRReviewService::new(
            repository,
            mock_executor.clone() as Arc<dyn CommandExecutor>,
        );

        (service, mock_executor)
    }

    #[tokio::test]
    async fn test_get_pr_info() {
        let (service, mock_executor) = setup().await;

        // Set up mock response
        let pr_json = r#"{
            "title": "Add new feature",
            "body": "This PR adds a new feature",
            "headRefName": "feature-branch",
            "baseRefName": "main",
            "author": {"login": "user123"},
            "state": "OPEN"
        }"#;

        mock_executor.add_response("gh", vec!["pr", "view", "123"], pr_json, "", true);

        let pr_info = service
            .get_pr_info("https://github.com/owner/repo/pull/123")
            .await
            .unwrap();

        assert_eq!(pr_info.pr_number, 123);
        assert_eq!(pr_info.title, "Add new feature");
        assert_eq!(pr_info.branch, "feature-branch");
        assert_eq!(pr_info.base_branch, "main");
        assert_eq!(pr_info.author, "user123");
        assert_eq!(pr_info.status, PRStatus::Open);
    }

    #[tokio::test]
    async fn test_review_pr_all_passing() {
        let (service, mock_executor) = setup().await;

        // Set up mock responses
        let pr_json = r#"{
            "title": "Fix bug",
            "body": "Fixes issue #42",
            "headRefName": "bugfix",
            "baseRefName": "main",
            "author": {"login": "developer"},
            "state": "OPEN"
        }"#;

        mock_executor.add_response("gh", vec!["pr", "view", "456"], pr_json, "", true);
        mock_executor.add_response(
            "git",
            vec!["checkout", "bugfix"],
            "Switched to branch 'bugfix'",
            "",
            true,
        );
        mock_executor.add_response(
            "cargo",
            vec!["test"],
            "test result: ok. 100 passed",
            "",
            true,
        );
        mock_executor.add_response(
            "git",
            vec!["checkout", "-"],
            "Switched to branch 'main'",
            "",
            true,
        );
        mock_executor.add_response(
            "git",
            vec!["merge", "--no-commit", "--no-ff", "bugfix"],
            "Automatic merge went well",
            "",
            true,
        );
        mock_executor.add_response("git", vec!["merge", "--abort"], "", "", true);

        let review = service
            .review_pr("https://github.com/owner/repo/pull/456".to_string())
            .await
            .unwrap();

        assert!(review.approved);
        assert!(review.tests_passed);
        assert!(!review.merge_conflicts);
        assert!(
            review
                .comments
                .iter()
                .any(|c| c.contains("All tests passed"))
        );
        assert!(review.comments.iter().any(|c| c.contains("ready to merge")));
    }

    #[tokio::test]
    async fn test_review_pr_tests_fail() {
        let (service, mock_executor) = setup().await;

        // Set up mock responses
        let pr_json = r#"{
            "title": "New feature",
            "body": "Adds feature X",
            "headRefName": "feature-x",
            "baseRefName": "main",
            "author": {"login": "dev"},
            "state": "OPEN"
        }"#;

        mock_executor.add_response("gh", vec!["pr", "view", "789"], pr_json, "", true);
        mock_executor.add_response(
            "git",
            vec!["checkout", "feature-x"],
            "Switched to branch 'feature-x'",
            "",
            true,
        );
        mock_executor.add_response(
            "cargo",
            vec!["test"],
            "",
            "test result: FAILED. 2 failed",
            false,
        );
        mock_executor.add_response(
            "git",
            vec!["checkout", "-"],
            "Switched to branch 'main'",
            "",
            true,
        );
        mock_executor.add_response(
            "git",
            vec!["merge", "--no-commit", "--no-ff", "feature-x"],
            "Automatic merge went well",
            "",
            true,
        );
        mock_executor.add_response("git", vec!["merge", "--abort"], "", "", true);

        let review = service
            .review_pr("https://github.com/owner/repo/pull/789".to_string())
            .await
            .unwrap();

        assert!(!review.approved);
        assert!(!review.tests_passed);
        assert!(!review.merge_conflicts);
        assert!(review.comments.iter().any(|c| c.contains("Tests failed")));
    }

    #[tokio::test]
    async fn test_review_pr_merge_conflicts() {
        let (service, mock_executor) = setup().await;

        // Set up mock responses
        let pr_json = r#"{
            "title": "Conflicting change",
            "body": "This might conflict",
            "headRefName": "conflict-branch",
            "baseRefName": "main",
            "author": {"login": "dev"},
            "state": "OPEN"
        }"#;

        mock_executor.add_response("gh", vec!["pr", "view", "999"], pr_json, "", true);
        mock_executor.add_response(
            "git",
            vec!["checkout", "conflict-branch"],
            "Switched to branch",
            "",
            true,
        );
        mock_executor.add_response("cargo", vec!["test"], "test result: ok", "", true);
        mock_executor.add_response(
            "git",
            vec!["checkout", "-"],
            "Switched to branch 'main'",
            "",
            true,
        );
        mock_executor.add_response(
            "git",
            vec!["merge", "--no-commit", "--no-ff", "conflict-branch"],
            "",
            "CONFLICT: Merge conflict in file.rs",
            false,
        );
        mock_executor.add_response("git", vec!["merge", "--abort"], "", "", true);

        let review = service
            .review_pr("https://github.com/owner/repo/pull/999".to_string())
            .await
            .unwrap();

        assert!(!review.approved);
        assert!(review.tests_passed);
        assert!(review.merge_conflicts);
        assert!(
            review
                .comments
                .iter()
                .any(|c| c.contains("Merge conflicts"))
        );
    }

    #[tokio::test]
    async fn test_approve_and_merge_success() {
        let (service, mock_executor) = setup().await;

        // Set up mock responses for successful review
        let pr_json = r#"{
            "title": "Ready to merge",
            "body": "All good",
            "headRefName": "ready",
            "baseRefName": "main",
            "author": {"login": "dev"},
            "state": "OPEN"
        }"#;

        mock_executor.add_response("gh", vec!["pr", "view", "111"], pr_json, "", true);
        mock_executor.add_response("git", vec!["checkout", "ready"], "Switched", "", true);
        mock_executor.add_response("cargo", vec!["test"], "test result: ok", "", true);
        mock_executor.add_response("git", vec!["checkout", "-"], "Switched", "", true);
        mock_executor.add_response(
            "git",
            vec!["merge", "--no-commit", "--no-ff", "ready"],
            "OK",
            "",
            true,
        );
        mock_executor.add_response("git", vec!["merge", "--abort"], "", "", true);
        mock_executor.add_response(
            "gh",
            vec!["pr", "review", "111", "--approve"],
            "Approved",
            "",
            true,
        );
        mock_executor.add_response(
            "gh",
            vec!["pr", "merge", "111", "--squash"],
            "Merged",
            "",
            true,
        );

        service
            .approve_and_merge("https://github.com/owner/repo/pull/111".to_string())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_approve_and_merge_fails_when_tests_fail() {
        let (service, mock_executor) = setup().await;

        // Set up mock responses with failing tests
        let pr_json = r#"{
            "title": "Broken PR",
            "body": "Has issues",
            "headRefName": "broken",
            "baseRefName": "main",
            "author": {"login": "dev"},
            "state": "OPEN"
        }"#;

        mock_executor.add_response("gh", vec!["pr", "view", "222"], pr_json, "", true);
        mock_executor.add_response("git", vec!["checkout", "broken"], "Switched", "", true);
        mock_executor.add_response("cargo", vec!["test"], "", "test FAILED", false);
        mock_executor.add_response("git", vec!["checkout", "-"], "Switched", "", true);
        mock_executor.add_response(
            "git",
            vec!["merge", "--no-commit", "--no-ff", "broken"],
            "OK",
            "",
            true,
        );
        mock_executor.add_response("git", vec!["merge", "--abort"], "", "", true);

        let result = service
            .approve_and_merge("https://github.com/owner/repo/pull/222".to_string())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be merged"));
    }
}
