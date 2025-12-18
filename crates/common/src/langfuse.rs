//! Langfuse integration for prompt management.
//!
//! Uses Langfuse as primary prompt source with TOML config as fallback.
//!
//! ## Environment Variables
//! - `LANGFUSE_PUBLIC_KEY`: Langfuse public key
//! - `LANGFUSE_SECRET_KEY`: Langfuse secret key
//! - `LANGFUSE_BASE_URL`: Langfuse API URL (default: https://cloud.langfuse.com)
//!
//! ## Usage
//! ```rust,ignore
//! let manager = LangfusePromptManager::new().await?;
//! let prompt = manager.get_prompt("sales-agent", Some("th"), Some(1)).await?;
//! let compiled = manager.compile(&prompt, &vars);
//! ```

use crate::prompt_config::PromptConfig;
use langfuse_ergonomic::client::{ClientBuilder, LangfuseClient};
use langfuse_ergonomic::Prompt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Compiled prompt ready for use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPrompt {
    pub name: String,
    pub version: Option<i32>,
    pub prompt: String,
    pub config: Option<serde_json::Value>,
    pub labels: Vec<String>,
    pub source: PromptSource,
}

/// Source of the prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptSource {
    Langfuse,
    TomlConfig,
    Default,
}

/// Langfuse prompt manager with caching and fallback
pub struct LangfusePromptManager {
    client: Option<LangfuseClient>,
    fallback_config: PromptConfig,
    cache: Arc<RwLock<HashMap<String, CompiledPrompt>>>,
}

// Helper functions for cleaner code
fn build_cache_key(name: &str, language: Option<&str>, version: Option<i32>) -> String {
    format!(
        "{}:{}:{}",
        name,
        language.unwrap_or("default"),
        version
            .map(|v| v.to_string())
            .unwrap_or_else(|| "latest".to_string())
    )
}

fn build_prompt_name(name: &str, language: Option<&str>) -> String {
    match language {
        Some(lang) => format!("{}-{}", name, lang),
        None => name.to_string(),
    }
}

impl LangfusePromptManager {
    /// Create a new prompt manager
    ///
    /// Tries to connect to Langfuse if credentials are available,
    /// otherwise uses TOML config as primary source.
    pub async fn new() -> crate::Result<Self> {
        Self::with_config(PromptConfig::load()).await
    }

    /// Create with custom fallback config
    pub async fn with_config(fallback_config: PromptConfig) -> crate::Result<Self> {
        let client = Self::try_create_client();

        if client.is_some() {
            tracing::info!("Langfuse client initialized successfully");
        } else {
            tracing::info!("Langfuse credentials not found, using TOML config only");
        }

        Ok(Self {
            client,
            fallback_config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Try to create Langfuse client from environment
    fn try_create_client() -> Option<LangfuseClient> {
        dotenvy::dotenv().ok();

        let public_key = std::env::var("LANGFUSE_PUBLIC_KEY").ok()?;
        let secret_key = std::env::var("LANGFUSE_SECRET_KEY").ok()?;
        let base_url = std::env::var("LANGFUSE_BASE_URL")
            .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string());

        match ClientBuilder::new()
            .public_key(&public_key)
            .secret_key(&secret_key)
            .base_url(&base_url)
            .build()
        {
            Ok(client) => Some(client),
            Err(e) => {
                tracing::warn!("Failed to create Langfuse client: {}", e);
                None
            }
        }
    }

    /// Check if Langfuse is available
    pub fn has_langfuse(&self) -> bool {
        self.client.is_some()
    }

    /// Get prompt by name with optional language suffix and version
    ///
    /// Tries Langfuse first, then falls back to TOML config.
    ///
    /// # Arguments
    /// * `name` - Prompt name (e.g., "sales-agent")
    /// * `language` - Optional language code (e.g., "th", "en")
    /// * `version` - Optional specific version
    pub async fn get_prompt(
        &self,
        name: &str,
        language: Option<&str>,
        version: Option<i32>,
    ) -> crate::Result<CompiledPrompt> {
        let cache_key = build_cache_key(name, language, version);

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Try Langfuse
        if let Some(client) = &self.client {
            let prompt_name = build_prompt_name(name, language);

            match self
                .fetch_from_langfuse(client, &prompt_name, version)
                .await
            {
                Ok(prompt) => {
                    let mut cache = self.cache.write().await;
                    cache.insert(cache_key, prompt.clone());
                    return Ok(prompt);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch prompt '{}' from Langfuse: {}, falling back to config",
                        prompt_name,
                        e
                    );
                }
            }
        }

        // Fallback to TOML config
        self.get_from_config(name, language).await
    }

    /// Fetch prompt from Langfuse API
    async fn fetch_from_langfuse(
        &self,
        client: &LangfuseClient,
        name: &str,
        version: Option<i32>,
    ) -> crate::Result<CompiledPrompt> {
        let response = client
            .get_prompt(name, version, None)
            .await
            .map_err(|e| crate::Error::Config(format!("Langfuse API error: {}", e)))?;

        // Extract prompt content from response
        let prompt_content = self.extract_prompt_content(&response)?;

        Ok(CompiledPrompt {
            name: name.to_string(),
            version: None, // Version info might not be directly available
            prompt: prompt_content,
            config: None,
            labels: vec![],
            source: PromptSource::Langfuse,
        })
    }

    /// Extract prompt content from Langfuse Prompt enum
    fn extract_prompt_content(&self, prompt: &Prompt) -> crate::Result<String> {
        match prompt {
            Prompt::PromptOneOf(chat_prompt) => {
                // Chat prompt type - serialize messages to JSON and extract content
                let json = serde_json::to_value(&chat_prompt.prompt)
                    .map_err(|e| crate::Error::Config(e.to_string()))?;

                if let serde_json::Value::Array(messages) = json {
                    let parts: Vec<String> = messages
                        .iter()
                        .filter_map(|msg| {
                            msg.get("content")
                                .and_then(|c| c.as_str())
                                .map(String::from)
                        })
                        .collect();
                    Ok(parts.join("\n\n"))
                } else {
                    Ok(json.to_string())
                }
            }
            Prompt::PromptOneOf1(text_prompt) => {
                // Text prompt type - it's a simple string
                Ok(text_prompt.prompt.clone())
            }
        }
    }

    /// Get prompt from TOML config (fallback)
    async fn get_from_config(
        &self,
        name: &str,
        language: Option<&str>,
    ) -> crate::Result<CompiledPrompt> {
        // Try as agent prompt first
        if let Some(prompt) = self
            .fallback_config
            .get_agent_prompt(name, language.unwrap_or("default"))
        {
            return Ok(CompiledPrompt {
                name: name.to_string(),
                version: None,
                prompt: prompt.to_string(),
                config: None,
                labels: vec![],
                source: PromptSource::TomlConfig,
            });
        }

        // Try as template
        if let Some(template) = self.fallback_config.get_template(name) {
            return Ok(CompiledPrompt {
                name: name.to_string(),
                version: None,
                prompt: template.prompt.clone(),
                config: None,
                labels: vec![],
                source: PromptSource::TomlConfig,
            });
        }

        Err(crate::Error::Config(format!(
            "Prompt '{}' not found in Langfuse or config",
            name
        )))
    }

    /// Compile prompt with variables
    ///
    /// Replaces `{{variable}}` placeholders with provided values.
    pub fn compile(&self, prompt: &CompiledPrompt, variables: &HashMap<String, String>) -> String {
        let mut result = prompt.prompt.clone();

        for (key, value) in variables {
            // Support both {{var}} and {var} formats
            result = result.replace(&format!("{{{{{}}}}}", key), value);
            result = result.replace(&format!("{{{}}}", key), value);
        }

        result
    }

    /// Clear the prompt cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::debug!("Prompt cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, Vec<String>) {
        let cache = self.cache.read().await;
        let keys: Vec<String> = cache.keys().cloned().collect();
        (cache.len(), keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_manager_fallback() {
        let manager = LangfusePromptManager::new().await.unwrap();

        // Should fall back to config since no Langfuse credentials
        let result = manager.get_prompt("sales", Some("th"), None).await;
        assert!(result.is_ok());

        let prompt = result.unwrap();
        assert_eq!(prompt.source, PromptSource::TomlConfig);
    }

    #[test]
    fn test_compile_prompt() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = rt.block_on(async {
            LangfusePromptManager::with_config(PromptConfig::default())
                .await
                .unwrap()
        });

        let prompt = CompiledPrompt {
            name: "test".to_string(),
            version: None,
            prompt: "Hello {{name}}, welcome to {{company}}!".to_string(),
            config: None,
            labels: vec![],
            source: PromptSource::Default,
        };

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "John".to_string());
        vars.insert("company".to_string(), "Acme".to_string());

        let compiled = manager.compile(&prompt, &vars);
        assert_eq!(compiled, "Hello John, welcome to Acme!");
    }
}
