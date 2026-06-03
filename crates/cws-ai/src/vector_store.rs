use std::collections::HashMap;
use std::sync::Mutex;

use tracing::info;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::SearchResult;
use cws_core::traits::VectorStore;

/// In-memory vector store using a HashMap.
/// TODO: Replace with LanceDB for persistent, scalable vector storage.
///
/// When integrating LanceDB:
/// - Use lancedb crate to create/open a Lance database
/// - Store vectors in a Lance table with schema (id, entity_type, vector)
/// - Use Lance's built-in ANN search for fast similarity queries
pub struct InMemoryVectorStore {
    store: Mutex<HashMap<String, VectorEntry>>,
}

struct VectorEntry {
    entity_type: String,
    embedding: Vec<f32>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        info!("In-memory vector store initialized");
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore for InMemoryVectorStore {
    fn insert(&self, entity_id: &str, entity_type: &str, embedding: &[f32]) -> CwsResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| CwsError::AiEngine(format!("Lock error: {}", e)))?;

        store.insert(
            entity_id.to_string(),
            VectorEntry {
                entity_type: entity_type.to_string(),
                embedding: embedding.to_vec(),
            },
        );

        Ok(())
    }

    fn search(&self, query_embedding: &[f32], limit: usize) -> CwsResult<Vec<SearchResult>> {
        let store = self
            .store
            .lock()
            .map_err(|e| CwsError::AiEngine(format!("Lock error: {}", e)))?;

        let mut scored: Vec<(String, String, f32)> = store
            .iter()
            .map(|(id, entry)| {
                let score = cosine_similarity(query_embedding, &entry.embedding);
                (id.clone(), entry.entity_type.clone(), score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored
            .into_iter()
            .map(|(id, entity_type, score)| SearchResult {
                id: id.clone(),
                entity_type,
                title: id, // Stub: use ID as title
                snippet: String::new(),
                score,
            })
            .collect())
    }

    fn delete(&self, entity_id: &str) -> CwsResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| CwsError::AiEngine(format!("Lock error: {}", e)))?;

        store.remove(entity_id);
        Ok(())
    }

    fn update(&self, entity_id: &str, entity_type: &str, embedding: &[f32]) -> CwsResult<()> {
        self.insert(entity_id, entity_type, embedding)
    }
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}
