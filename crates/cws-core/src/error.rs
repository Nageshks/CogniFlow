use thiserror::Error;

/// Errors for the CogniFlow system.
#[derive(Error, Debug)]
pub enum CwsError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("AI engine error: {0}")]
    AiEngine(String),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Import/Export error: {0}")]
    ImportExport(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Folder depth exceeds maximum of {max}")]
    FolderDepthExceeded { max: i32 },

    #[error("Model not found. Please download the AI model first.")]
    ModelNotFound,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for CogniFlow operations.
pub type CwsResult<T> = Result<T, CwsError>;

/// Convert CwsError to a string for Tauri IPC serialization.
impl serde::Serialize for CwsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
