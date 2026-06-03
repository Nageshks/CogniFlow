use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A folder for organizing prompts. Supports nesting up to 10 levels deep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub depth: i32,
}

impl Folder {
    pub fn new(name: String, parent_id: Option<Uuid>, depth: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            parent_id,
            depth,
        }
    }
}

/// Maximum nesting depth for folders.
pub const MAX_FOLDER_DEPTH: i32 = 10;

/// Folder tree node for UI rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTreeNode {
    #[serde(flatten)]
    pub folder: Folder,
    pub children: Vec<FolderTreeNode>,
    pub prompt_count: i32,
}
