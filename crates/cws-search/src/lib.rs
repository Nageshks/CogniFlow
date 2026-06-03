mod indexer;
mod searcher;

use std::path::Path;

use tantivy::Index;
use tracing::info;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::SearchResult;
use cws_core::traits::FullTextSearch;

use indexer::Indexer;
use searcher::Searcher;

/// Full-text search engine backed by Tantivy.
pub struct SearchEngine {
    indexer: Indexer,
    searcher: Searcher,
}

impl SearchEngine {
    /// Create or open an existing Tantivy index at the given directory path.
    pub fn new(index_path: &Path) -> CwsResult<Self> {
        std::fs::create_dir_all(index_path).map_err(|e| {
            CwsError::Search(format!("Failed to create index directory: {}", e))
        })?;

        let schema = indexer::build_schema();

        let index = if index_path.join("meta.json").exists() {
            Index::open_in_dir(index_path).map_err(|e| CwsError::Search(e.to_string()))?
        } else {
            Index::create_in_dir(index_path, schema.schema.clone())
                .map_err(|e| CwsError::Search(e.to_string()))?
        };

        let indexer = Indexer::new(index.clone(), schema.clone())?;
        let searcher = Searcher::new(index, schema)?;

        info!("Search engine initialized at {:?}", index_path);
        Ok(Self { indexer, searcher })
    }

    /// Create a search engine with an in-memory (RAM) index for testing.
    pub fn new_in_memory() -> CwsResult<Self> {
        let schema = indexer::build_schema();
        let index = Index::create_in_ram(schema.schema.clone());

        let indexer = Indexer::new(index.clone(), schema.clone())?;
        let searcher = Searcher::new(index, schema)?;

        info!("In-memory search engine initialized");
        Ok(Self { indexer, searcher })
    }
}

impl FullTextSearch for SearchEngine {
    fn index_document(
        &self,
        id: &str,
        entity_type: &str,
        title: &str,
        content: &str,
        description: &str,
    ) -> CwsResult<()> {
        // Remove existing document first (re-index)
        let _ = self.indexer.remove_document(id);
        self.indexer
            .add_document(id, entity_type, title, content, description)?;
        self.indexer.commit()
    }

    fn remove_document(&self, id: &str) -> CwsResult<()> {
        self.indexer.remove_document(id)?;
        self.indexer.commit()
    }

    fn search(&self, query: &str, limit: usize) -> CwsResult<Vec<SearchResult>> {
        self.searcher.search(query, limit)
    }
}
