use crate::error::CwsResult;
use crate::models::SearchResult;

/// Abstraction over vector storage backends (LanceDB now, potentially Qdrant later).
pub trait VectorStore: Send + Sync {
    /// Store an embedding vector for an entity.
    fn insert(&self, entity_id: &str, entity_type: &str, embedding: &[f32]) -> CwsResult<()>;

    /// Search for the top N most similar entities to a query vector.
    fn search(&self, query_embedding: &[f32], limit: usize) -> CwsResult<Vec<SearchResult>>;

    /// Delete the embedding for an entity.
    fn delete(&self, entity_id: &str) -> CwsResult<()>;

    /// Update an existing embedding.
    fn update(&self, entity_id: &str, entity_type: &str, embedding: &[f32]) -> CwsResult<()>;
}

/// Abstraction over the full-text search engine (Tantivy now).
pub trait FullTextSearch: Send + Sync {
    /// Index a document for full-text search.
    fn index_document(&self, id: &str, entity_type: &str, title: &str, content: &str, description: &str) -> CwsResult<()>;

    /// Remove a document from the index.
    fn remove_document(&self, id: &str) -> CwsResult<()>;

    /// Search for documents matching a query string.
    fn search(&self, query: &str, limit: usize) -> CwsResult<Vec<SearchResult>>;
}

/// Abstraction over the embedding generation engine.
pub trait EmbeddingEngine: Send + Sync {
    /// Generate an embedding vector for the given text.
    fn embed(&self, text: &str) -> CwsResult<Vec<f32>>;

    /// Generate embeddings for multiple texts in a batch.
    fn embed_batch(&self, texts: &[&str]) -> CwsResult<Vec<Vec<f32>>>;

    /// Get the dimensionality of the embedding vectors.
    fn dimensions(&self) -> usize;
}
