use anyhow::Result;
use async_trait::async_trait;

/// Trait for embedding providers (OpenAI, Ollama, Gemini, etc.)
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a text string
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Get the embedding dimensions
    fn dimensions(&self) -> usize;
}

/// Create an embedding provider based on config
pub fn create_provider(provider: &str, model: Option<&str>) -> Result<Box<dyn EmbeddingProvider>> {
    match provider {
        "openai" => {
            let model = model.unwrap_or("text-embedding-3-small");
            Ok(Box::new(crate::embedding::openai::OpenAIProvider::new(model)?))
        }
        "none" => {
            anyhow::bail!("Embedding provider is set to 'none'. Configure with: am init --embedding openai")
        }
        _ => {
            anyhow::bail!("Unknown embedding provider: '{}'. Supported: openai", provider)
        }
    }
}
