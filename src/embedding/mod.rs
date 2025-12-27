pub mod openai;
pub mod qdrant;
pub mod service;

pub use service::{create_provider, EmbeddingProvider};
