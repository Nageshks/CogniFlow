pub mod embeddings;
pub mod vector_store;
pub mod classifier;
pub mod similarity;

use cws_core::error::CwsResult;
use cws_core::models::SearchResult;

use embeddings::StubEmbeddingEngine;
use vector_store::InMemoryVectorStore;

/// Main AI engine combining embeddings, vector store, and classification.
pub struct AiEngine {
    pub embedding_engine: StubEmbeddingEngine,
    pub vector_store: InMemoryVectorStore,
}

impl AiEngine {
    /// Create a new AI engine with stub implementations.
    pub fn new() -> CwsResult<Self> {
        Ok(Self {
            embedding_engine: StubEmbeddingEngine::new(),
            vector_store: InMemoryVectorStore::new(),
        })
    }

    /// Search for similar entities using semantic search.
    pub fn semantic_search(&self, query: &str, limit: usize) -> CwsResult<Vec<SearchResult>> {
        use cws_core::traits::EmbeddingEngine;
        use cws_core::traits::VectorStore;

        let query_embedding = self.embedding_engine.embed(query)?;
        self.vector_store.search(&query_embedding, limit)
    }

    /// Index an entity for semantic search.
    pub fn index_entity(&self, entity_id: &str, entity_type: &str, text: &str) -> CwsResult<()> {
        use cws_core::traits::EmbeddingEngine;
        use cws_core::traits::VectorStore;

        let embedding = self.embedding_engine.embed(text)?;
        self.vector_store.insert(entity_id, entity_type, &embedding)
    }

    /// Remove an entity from semantic search.
    pub fn remove_entity(&self, entity_id: &str) -> CwsResult<()> {
        use cws_core::traits::VectorStore;
        self.vector_store.delete(entity_id)
    }
}

impl Default for AiEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default AiEngine")
    }
}
