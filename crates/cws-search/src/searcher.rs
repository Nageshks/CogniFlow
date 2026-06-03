use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::{Index, ReloadPolicy};

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::SearchResult;

use crate::indexer::SearchSchema;

/// Handles searching the Tantivy index.
pub struct Searcher {
    reader: tantivy::IndexReader,
    query_parser: QueryParser,
    fields: SearchSchema,
}

impl Searcher {
    pub fn new(index: Index, fields: SearchSchema) -> CwsResult<Self> {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e: tantivy::TantivyError| CwsError::Search(e.to_string()))?;

        let query_parser = QueryParser::for_index(
            &index,
            vec![fields.title, fields.content, fields.description],
        );

        Ok(Self {
            reader,
            query_parser,
            fields,
        })
    }

    /// Search for documents matching the query string.
    pub fn search(&self, query_str: &str, limit: usize) -> CwsResult<Vec<SearchResult>> {
        let searcher = self.reader.searcher();

        let query = self
            .query_parser
            .parse_query(query_str)
            .map_err(|e| CwsError::Search(format!("Failed to parse query: {}", e)))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| CwsError::Search(format!("Search failed: {}", e)))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address).map_err(|e| {
                CwsError::Search(format!("Failed to retrieve document: {}", e))
            })?;

            let id = doc
                .get_first(self.fields.id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let entity_type = doc
                .get_first(self.fields.entity_type)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = doc
                .get_first(self.fields.title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Use title as snippet since content is not stored
            let snippet = title.clone();

            results.push(SearchResult {
                id,
                entity_type,
                title,
                snippet,
                score,
            });
        }

        Ok(results)
    }
}
