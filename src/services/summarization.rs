use crate::domain::goal::Goal;
use crate::domain::task::Task;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SummarizationLevel {
    HighLevel, // Very abstract, 1-2 sentences
    MidLevel,  // Moderate detail, 3-4 sentences
    LowLevel,  // More detail, 5-6 sentences
    Detailed,  // Full information
}

#[derive(Debug, Clone)]
pub struct SummaryRequest {
    pub content: String,
    pub level: SummarizationLevel,
    pub context: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SummaryResponse {
    pub summary: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

#[derive(Clone)]
pub struct SummarizationService {
    cache: Arc<RwLock<SummaryCache>>,
    model_endpoint: String,
    api_key: Option<String>,
    max_retries: usize,
    timeout: Duration,
}

pub struct SummaryCache {
    entries: HashMap<CacheKey, CacheEntry>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    content_hash: u64,
    level: SummarizationLevel,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    summary: String,
    created_at: Instant,
    access_count: usize,
}

impl Default for SummarizationService {
    fn default() -> Self {
        Self::new()
    }
}

impl SummarizationService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(SummaryCache::new(500))),
            model_endpoint: Self::get_model_endpoint(),
            api_key: Self::get_api_key(),
            max_retries: 3,
            timeout: Duration::from_secs(5),
        }
    }

    fn get_model_endpoint() -> String {
        std::env::var("LLM_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string())
    }

    fn get_api_key() -> Option<String> {
        std::env::var("LLM_API_KEY").ok()
    }

    pub async fn summarize(&self, content: &str, level: SummarizationLevel) -> String {
        let key = CacheKey {
            content_hash: Self::hash_content(content),
            level,
        };

        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(summary) = cache.get(&key) {
                return summary.to_string();
            }
        }

        // Generate summary
        let summary = self
            .generate_summary(content, level)
            .await
            .unwrap_or_else(|_| self.fallback_summary(content, level));

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(key, summary.clone());
        }

        summary
    }

    pub async fn summarize_with_cache(
        &self,
        cache: &mut SummaryCache,
        id: Uuid,
        content: &str,
        level: SummarizationLevel,
    ) -> String {
        let key = CacheKey {
            content_hash: Self::hash_content(&format!("{}{}", id, content)),
            level,
        };

        if let Some(summary) = cache.get(&key) {
            return summary;
        }

        let summary = self
            .generate_summary(content, level)
            .await
            .unwrap_or_else(|_| self.fallback_summary(content, level));

        cache.insert(key, summary.clone());
        summary
    }

    pub async fn summarize_cluster(&self, tasks: &[Task], level: SummarizationLevel) -> String {
        if tasks.is_empty() {
            return String::new();
        }

        // Combine task information
        let combined_content = tasks
            .iter()
            .map(|t| format!("{}: {}", t.title, t.description))
            .collect::<Vec<_>>()
            .join("\n");

        // Generate cluster summary with context
        let context = format!("This is a cluster of {} related tasks", tasks.len());
        self.summarize_with_context(&combined_content, level, &context)
            .await
    }

    pub async fn summarize_goal(
        &self,
        goal: &Goal,
        tasks: &[Task],
        level: SummarizationLevel,
    ) -> String {
        let mut content = format!("Goal: {}\n{}", goal.title, goal.description);

        if !tasks.is_empty() {
            content.push_str("\n\nAssociated tasks:\n");
            for task in tasks {
                content.push_str(&format!("- {}: {}\n", task.title, task.description));
            }
        }

        let context = "Summarize this project goal and its tasks";
        self.summarize_with_context(&content, level, context).await
    }

    async fn summarize_with_context(
        &self,
        content: &str,
        level: SummarizationLevel,
        context: &str,
    ) -> String {
        let prompt = self.build_prompt(content, level, Some(context));

        match self.call_llm(&prompt).await {
            Ok(response) => response,
            Err(_) => self.fallback_summary(content, level),
        }
    }

    async fn generate_summary(&self, content: &str, level: SummarizationLevel) -> Result<String> {
        let prompt = self.build_prompt(content, level, None);
        self.call_llm(&prompt).await
    }

    fn build_prompt(
        &self,
        content: &str,
        level: SummarizationLevel,
        context: Option<&str>,
    ) -> String {
        let level_instruction = match level {
            SummarizationLevel::HighLevel => {
                "Provide an extremely concise 1-2 sentence summary capturing only the essential point."
            }
            SummarizationLevel::MidLevel => {
                "Provide a brief 3-4 sentence summary with key details."
            }
            SummarizationLevel::LowLevel => {
                "Provide a comprehensive 5-6 sentence summary including important details."
            }
            SummarizationLevel::Detailed => {
                "Provide a detailed summary preserving all important information."
            }
        };

        let context_str = context.unwrap_or("Summarize the following content");

        format!(
            "{}\n\n{}\n\nContent:\n{}\n\nSummary:",
            context_str, level_instruction, content
        )
    }

    async fn call_llm(&self, prompt: &str) -> Result<String> {
        // For Ollama integration
        if self.model_endpoint.contains("ollama") || self.model_endpoint.contains("11434") {
            return self.call_ollama(prompt).await;
        }

        // For OpenAI-compatible APIs
        if let Some(api_key) = &self.api_key {
            return self.call_openai_compatible(prompt, api_key).await;
        }

        // Fallback to local processing
        Ok(self.fallback_summary(prompt, SummarizationLevel::MidLevel))
    }

    async fn call_ollama(&self, prompt: &str) -> Result<String> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
            options: OllamaOptions,
        }

        #[derive(Serialize)]
        struct OllamaOptions {
            temperature: f32,
            top_p: f32,
            max_tokens: i32,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let request = OllamaRequest {
            model: "llama3.2:1b".to_string(), // Fast small model
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: 0.3,
                top_p: 0.9,
                max_tokens: 150,
            },
        };

        let client = reqwest::Client::new();
        let response = client
            .post(&self.model_endpoint)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await?;

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }

    async fn call_openai_compatible(&self, prompt: &str, api_key: &str) -> Result<String> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
            max_tokens: i32,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<Choice>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: ResponseMessage,
        }

        #[derive(Deserialize)]
        struct ResponseMessage {
            content: String,
        }

        let request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(), // Fast model
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a concise summarization assistant.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 150,
        };

        let client = reqwest::Client::new();
        let response = client
            .post(&self.model_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await?;

        let openai_response: OpenAIResponse = response.json().await?;
        Ok(openai_response.choices[0].message.content.clone())
    }

    fn fallback_summary(&self, content: &str, level: SummarizationLevel) -> String {
        // Simple rule-based summarization as fallback
        let sentences: Vec<&str> = content
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .collect();

        if sentences.is_empty() {
            return content.chars().take(50).collect();
        }

        let max_sentences = match level {
            SummarizationLevel::HighLevel => 1,
            SummarizationLevel::MidLevel => 2,
            SummarizationLevel::LowLevel => 3,
            SummarizationLevel::Detailed => sentences.len(),
        };

        let selected = sentences
            .iter()
            .take(max_sentences)
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(". ");

        if selected.len() > 200 && level != SummarizationLevel::Detailed {
            format!("{}...", &selected[..197])
        } else {
            selected
        }
    }

    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    pub fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            model_endpoint: self.model_endpoint.clone(),
            api_key: self.api_key.clone(),
            max_retries: self.max_retries,
            timeout: self.timeout,
        }
    }
}

impl SummaryCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            ttl: Duration::from_secs(900), // 15 minutes
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<String> {
        if let Some(entry) = self.entries.get_mut(key) {
            // Check if entry is still valid
            if entry.created_at.elapsed() < self.ttl {
                entry.access_count += 1;
                return Some(entry.summary.clone());
            }
        }
        // Remove expired entry if it exists
        self.entries.remove(key);
        None
    }

    pub fn insert(&mut self, key: CacheKey, summary: String) {
        // Evict old entries if at capacity
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }

        self.entries.insert(
            key,
            CacheEntry {
                summary,
                created_at: Instant::now(),
                access_count: 0,
            },
        );
    }

    fn evict_lru(&mut self) {
        // Find least recently used entry
        if let Some(lru_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| (entry.access_count, entry.created_at))
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&lru_key);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_summarize_caching() {
        let service = SummarizationService::new();
        let content = "This is a long text that needs to be summarized. It contains multiple sentences and paragraphs.";

        // First call should generate summary
        let summary1 = service
            .summarize(content, SummarizationLevel::HighLevel)
            .await;

        // Second call should use cache (same content and level)
        let summary2 = service
            .summarize(content, SummarizationLevel::HighLevel)
            .await;
        assert_eq!(summary1, summary2);

        // Different level should generate different summary
        let summary3 = service
            .summarize(content, SummarizationLevel::Detailed)
            .await;
        assert_ne!(summary1, summary3);
    }

    #[tokio::test]
    async fn test_summarize_task() {
        let service = SummarizationService::new();
        let mut task = Task::new(
            "Implement feature X".to_string(),
            "This task involves creating a new feature that allows users to export data in multiple formats.".to_string()
        );
        task.add_tag("backend".to_string());
        task.add_tag("export".to_string());

        // Use the generic summarize method with task content
        let content = format!("{}: {}", task.title, task.description);
        let summary = service
            .summarize(&content, SummarizationLevel::HighLevel)
            .await;
        assert!(!summary.is_empty());
        assert!(summary.len() <= content.len()); // Summary should not be longer
    }

    #[tokio::test]
    async fn test_summarize_goal() {
        let service = SummarizationService::new();
        let goal = Goal::new(
            "Q1 Objectives".to_string(),
            "Complete the migration to the new infrastructure and launch three new features."
                .to_string(),
        );

        // Use the generic summarize method with goal content
        let content = format!("{}: {}", goal.title, goal.description);
        let summary = service
            .summarize(&content, SummarizationLevel::MidLevel)
            .await;
        assert!(!summary.is_empty());
    }

    #[tokio::test]
    async fn test_batch_summarize() {
        let service = SummarizationService::new();
        let tasks = vec![
            Task::new("Task 1".to_string(), "Description 1".to_string()),
            Task::new("Task 2".to_string(), "Description 2".to_string()),
            Task::new("Task 3".to_string(), "Description 3".to_string()),
        ];

        // Summarize each task individually
        let mut summaries = Vec::new();
        for task in &tasks {
            let content = format!("{}: {}", task.title, task.description);
            let summary = service
                .summarize(&content, SummarizationLevel::HighLevel)
                .await;
            summaries.push(summary);
        }

        assert_eq!(summaries.len(), tasks.len());
        for summary in summaries.iter() {
            assert!(!summary.is_empty());
        }
    }

    #[test]
    fn test_summary_cache() {
        let mut cache = SummaryCache::new(2);

        let key1 = CacheKey {
            content_hash: 123,
            level: SummarizationLevel::HighLevel,
        };
        let key2 = CacheKey {
            content_hash: 456,
            level: SummarizationLevel::MidLevel,
        };
        let key3 = CacheKey {
            content_hash: 789,
            level: SummarizationLevel::LowLevel,
        };

        cache.insert(key1.clone(), "Summary 1".to_string());
        assert_eq!(cache.size(), 1);

        cache.insert(key2.clone(), "Summary 2".to_string());
        assert_eq!(cache.size(), 2);

        // Should evict LRU entry when at capacity
        cache.insert(key3.clone(), "Summary 3".to_string());
        assert_eq!(cache.size(), 2);

        // Access key2 to update its access count
        assert!(cache.get(&key2).is_some());

        // Clear cache
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_content_hash() {
        let content1 = "This is test content";
        let content2 = "This is test content"; // Same content
        let content3 = "Different content";

        let hash1 = SummarizationService::hash_content(content1);
        let hash2 = SummarizationService::hash_content(content2);
        let hash3 = SummarizationService::hash_content(content3);

        assert_eq!(hash1, hash2); // Same content should have same hash
        assert_ne!(hash1, hash3); // Different content should have different hash
    }

    #[test]
    fn test_summarization_levels() {
        // Test that different levels produce different summaries
        let levels = [
            (SummarizationLevel::HighLevel, "high"),
            (SummarizationLevel::MidLevel, "mid"),
            (SummarizationLevel::LowLevel, "low"),
            (SummarizationLevel::Detailed, "detailed"),
        ];

        // Just verify the levels are different
        for (i, (level1, _)) in levels.iter().enumerate() {
            for (j, (level2, _)) in levels.iter().enumerate() {
                if i != j {
                    assert_ne!(level1, level2);
                }
            }
        }
    }
}
