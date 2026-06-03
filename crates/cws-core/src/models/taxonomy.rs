use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A tag that can be applied to prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}

/// A category for organizing prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}

/// An intent classification for a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub id: Uuid,
    pub name: String,
}

impl Intent {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}

/// Result of AI intent classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub intent: String,
    pub confidence: f32,
}

/// AI-generated suggestions for a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestions {
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub intent: Option<IntentClassification>,
}
