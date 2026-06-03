use tantivy::schema::{Field, Schema, STORED, STRING, TEXT};
use tantivy::{Index, IndexWriter, Term};
use tracing::info;

use cws_core::error::{CwsError, CwsResult};

/// Schema field references for convenience.
#[derive(Clone)]
pub struct SearchSchema {
    pub schema: Schema,
    pub id: Field,
    pub entity_type: Field,
    pub title: Field,
    pub content: Field,
    pub description: Field,
}

/// Build the Tantivy schema for CogniFlow documents.
pub fn build_schema() -> SearchSchema {
    let mut schema_builder = Schema::builder();

    let id = schema_builder.add_text_field("id", STRING | STORED);
    let entity_type = schema_builder.add_text_field("entity_type", STRING | STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let content = schema_builder.add_text_field("content", TEXT);
    let description = schema_builder.add_text_field("description", TEXT);

    let schema = schema_builder.build();

    SearchSchema {
        schema,
        id,
        entity_type,
        title,
        content,
        description,
    }
}

/// Handles indexing documents into Tantivy.
pub struct Indexer {
    writer: std::sync::Mutex<IndexWriter>,
    fields: SearchSchema,
}

impl Indexer {
    pub fn new(_index: Index, fields: SearchSchema) -> CwsResult<Self> {
        let writer = _index
            .writer(50_000_000) // 50MB heap
            .map_err(|e| CwsError::Search(format!("Failed to create index writer: {}", e)))?;

        Ok(Self {
            writer: std::sync::Mutex::new(writer),
            fields,
        })
    }

    /// Add a document to the index.
    pub fn add_document(
        &self,
        id: &str,
        entity_type: &str,
        title: &str,
        content: &str,
        description: &str,
    ) -> CwsResult<()> {
        let writer = self.writer.lock().map_err(|e| {
            CwsError::Search(format!("Failed to lock index writer: {}", e))
        })?;

        let mut doc = tantivy::TantivyDocument::default();
        doc.add_text(self.fields.id, id);
        doc.add_text(self.fields.entity_type, entity_type);
        doc.add_text(self.fields.title, title);
        doc.add_text(self.fields.content, content);
        doc.add_text(self.fields.description, description);

        writer.add_document(doc).map_err(|e| {
            CwsError::Search(format!("Failed to add document: {}", e))
        })?;

        Ok(())
    }

    /// Remove a document from the index by its ID.
    pub fn remove_document(&self, id: &str) -> CwsResult<()> {
        let writer = self.writer.lock().map_err(|e| {
            CwsError::Search(format!("Failed to lock index writer: {}", e))
        })?;

        let term = Term::from_field_text(self.fields.id, id);
        writer.delete_term(term);

        Ok(())
    }

    /// Commit pending changes to the index.
    pub fn commit(&self) -> CwsResult<()> {
        let mut writer = self.writer.lock().map_err(|e| {
            CwsError::Search(format!("Failed to lock index writer: {}", e))
        })?;

        writer.commit().map_err(|e| {
            CwsError::Search(format!("Failed to commit index: {}", e))
        })?;

        info!("Search index committed");
        Ok(())
    }
}
