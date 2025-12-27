use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs, EmbeddingInput,
    },
    Client,
};
use async_trait::async_trait;

use crate::embedding::service::EmbeddingProvider;

/// OpenAI embedding provider
pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
    dimensions: usize,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    /// Reads API key from OPENAI_API_KEY environment variable
    pub fn new(model: &str) -> Result<Self> {
        // Check for API key
        std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?;

        let client = Client::new();

        // Determine dimensions based on model
        let dimensions = match model {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536, // Default
        };

        Ok(Self {
            client,
            model: model.to_string(),
            dimensions,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(EmbeddingInput::String(text.to_string()))
            .build()
            .context("Failed to build embedding request")?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .context("Failed to create embedding")?;

        let embedding = response
            .data
            .first()
            .context("No embedding returned")?
            .embedding
            .clone();

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let input: Vec<String> = texts.iter().map(|s| s.to_string()).collect();

        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(EmbeddingInput::StringArray(input))
            .build()
            .context("Failed to build batch embedding request")?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .context("Failed to create batch embeddings")?;

        let embeddings: Vec<Vec<f32>> = response
            .data
            .into_iter()
            .map(|e| e.embedding)
            .collect();

        Ok(embeddings)
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// Standalone chat completion function for use outside the provider
/// Reads API key from OPENAI_API_KEY environment variable
pub async fn chat_completion(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    // Check for API key
    std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable not set")?;

    let client: Client<OpenAIConfig> = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .context("Failed to build system message")?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_prompt)
                .build()
                .context("Failed to build user message")?
                .into(),
        ])
        .build()
        .context("Failed to build chat completion request")?;

    let response = client
        .chat()
        .create(request)
        .await
        .context("Failed to create chat completion")?;

    let content = response
        .choices
        .first()
        .context("No response from model")?
        .message
        .content
        .clone()
        .unwrap_or_default();

    Ok(content)
}
