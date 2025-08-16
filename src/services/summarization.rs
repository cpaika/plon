use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};
use crate::domain::task::Task;
use crate::domain::goal::Goal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SummarizationLevel {
    HighLevel,  // Very abstract, 1-2 sentences
    MidLevel,   // Moderate detail, 3-4 sentences  
    LowLevel,   // More detail, 5-6 sentences
    Detailed,   // Full information
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
struct CacheKey {
    content_hash: u64,
    level: SummarizationLevel,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    summary: String,
    created_at: Instant,
    access_count: usize,
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
        std::env::var("LLM_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string())
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
        let summary = self.generate_summary(content, level).await.unwrap_or_else(|_| {
            self.fallback_summary(content, level)
        });

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

        let summary = self.generate_summary(content, level).await.unwrap_or_else(|_| {
            self.fallback_summary(content, level)
        });

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
        self.summarize_with_context(&combined_content, level, &context).await
    }

    pub async fn summarize_goal(&self, goal: &Goal, tasks: &[Task], level: SummarizationLevel) -> String {
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

    async fn summarize_with_context(&self, content: &str, level: SummarizationLevel, context: &str) -> String {
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

    fn build_prompt(&self, content: &str, level: SummarizationLevel, context: Option<&str>) -> String {
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
            context_str,
            level_instruction,
            content
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