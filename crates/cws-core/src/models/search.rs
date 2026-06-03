use serde::{Deserialize, Serialize};

/// A single search result from any search engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub entity_type: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

/// Configuration for hybrid search ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    /// Weight for semantic (vector) search results. Default: 0.7
    pub semantic_weight: f32,
    /// Weight for keyword (full-text) search results. Default: 0.3
    pub keyword_weight: f32,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.7,
            keyword_weight: 0.3,
        }
    }
}

/// A search query with mode selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub mode: SearchMode,
    pub limit: usize,
    pub config: Option<HybridSearchConfig>,
}

/// Available search modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Keyword,
    Semantic,
    Hybrid,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Hybrid
    }
}

/// Export format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Markdown,
}

/// Import format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Json,
    Markdown,
    Txt,
}

/// Export request configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub path: String,
    /// For markdown workflow export: single file or expanded folder
    pub expanded: bool,
}

/// Import request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub format: ImportFormat,
    pub path: String,
}

/// Statistics about the data store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStats {
    pub total_prompts: i64,
    pub total_workflows: i64,
    pub total_tags: i64,
    pub total_folders: i64,
    pub total_relationships: i64,
    pub archived_prompts: i64,
}
