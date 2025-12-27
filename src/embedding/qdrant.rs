use anyhow::{Context, Result};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, PointStruct, SearchPoints,
    VectorParams, VectorsConfig, with_payload_selector::SelectorOptions, WithPayloadSelector,
};
use serde_json::json;
use std::collections::HashMap;

/// Qdrant vector store for memory embeddings
pub struct QdrantStore {
    client: QdrantClient,
    collection: String,
    dimensions: u64,
}

impl QdrantStore {
    /// Create a new Qdrant store connection
    pub async fn new(url: &str, collection: &str, dimensions: usize) -> Result<Self> {
        let client = QdrantClient::from_url(url)
            .build()
            .context("Failed to connect to Qdrant")?;

        let store = Self {
            client,
            collection: collection.to_string(),
            dimensions: dimensions as u64,
        };

        // Ensure collection exists
        store.init_collection().await?;

        Ok(store)
    }

    /// Initialize the collection if it doesn't exist
    async fn init_collection(&self) -> Result<()> {
        let collections = self
            .client
            .list_collections()
            .await
            .context("Failed to list collections")?;

        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection);

        if !exists {
            self.client
                .create_collection(&CreateCollection {
                    collection_name: self.collection.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(Config::Params(VectorParams {
                            size: self.dimensions,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                })
                .await
                .context("Failed to create collection")?;
        }

        Ok(())
    }

    /// Upsert a memory embedding into the store
    pub async fn upsert(
        &self,
        memory_id: &str,
        embedding: Vec<f32>,
        memory_type: &str,
        title: &str,
    ) -> Result<()> {
        let payload: Payload = json!({
            "memory_id": memory_id,
            "memory_type": memory_type,
            "title": title,
        })
        .try_into()
        .context("Failed to create payload")?;

        // Use a deterministic hash of memory_id as the point ID
        let point_id = uuid_to_u64(memory_id);

        let point = PointStruct::new(point_id, embedding, payload);

        self.client
            .upsert_points_blocking(&self.collection, None, vec![point], None)
            .await
            .context("Failed to upsert point")?;

        Ok(())
    }

    /// Search for similar memories
    pub async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let search_result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.collection.clone(),
                vector: query_embedding,
                limit: limit as u64,
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(SelectorOptions::Enable(true)),
                }),
                ..Default::default()
            })
            .await
            .context("Failed to search points")?;

        let results: Vec<SearchResult> = search_result
            .result
            .into_iter()
            .map(|p| {
                let payload = p.payload;
                SearchResult {
                    memory_id: get_string_from_payload(&payload, "memory_id"),
                    memory_type: get_string_from_payload(&payload, "memory_type"),
                    title: get_string_from_payload(&payload, "title"),
                    score: p.score,
                }
            })
            .collect();

        Ok(results)
    }

    /// Delete a memory embedding from the store
    pub async fn delete(&self, memory_id: &str) -> Result<()> {
        let point_id = uuid_to_u64(memory_id);

        self.client
            .delete_points_blocking(
                &self.collection,
                None,
                &vec![point_id.into()].into(),
                None,
            )
            .await
            .context("Failed to delete point")?;

        Ok(())
    }

    /// Check if the store is connected and healthy
    pub async fn health_check(&self) -> Result<bool> {
        self.client
            .health_check()
            .await
            .context("Qdrant health check failed")?;
        Ok(true)
    }
}

/// Result from a semantic search
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory_id: String,
    pub memory_type: String,
    pub title: String,
    pub score: f32,
}

/// Convert a UUID string to a u64 point ID using a simple hash
fn uuid_to_u64(uuid_str: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    uuid_str.hash(&mut hasher);
    hasher.finish()
}

/// Helper to extract string from Qdrant payload
fn get_string_from_payload(payload: &HashMap<String, qdrant_client::qdrant::Value>, key: &str) -> String {
    payload
        .get(key)
        .and_then(|v| {
            use qdrant_client::qdrant::value::Kind;
            match &v.kind {
                Some(Kind::StringValue(s)) => Some(s.clone()),
                _ => None,
            }
        })
        .unwrap_or_default()
}
