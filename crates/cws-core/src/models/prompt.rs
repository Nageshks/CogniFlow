use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A cognitive prompt — the primary asset in CogniFlow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub description: String,
    pub folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
    pub current_version: i32,
}

impl Prompt {
    pub fn new(title: String, content: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            description,
            folder_id: None,
            created_at: now,
            updated_at: now,
            archived: false,
            current_version: 1,
        }
    }
}

/// A version snapshot of a prompt's content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersion {
    pub id: Uuid,
    pub prompt_id: Uuid,
    pub version_number: i32,
    pub content: String,
    pub change_summary: String,
    pub created_at: DateTime<Utc>,
}

impl PromptVersion {
    pub fn new(prompt_id: Uuid, version_number: i32, content: String, change_summary: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            prompt_id,
            version_number,
            content,
            change_summary,
            created_at: Utc::now(),
        }
    }
}

/// Request to create a new prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub description: String,
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

/// Request to update an existing prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePromptRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
    pub change_summary: String,
}

/// Prompt with associated metadata for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptWithMeta {
    #[serde(flatten)]
    pub prompt: Prompt,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}
