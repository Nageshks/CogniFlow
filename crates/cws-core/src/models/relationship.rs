use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of relationships between cognitive assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    Similar,
    DerivedFrom,
    UsedIn,
    RelatedTo,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Similar => "SIMILAR",
            Self::DerivedFrom => "DERIVED_FROM",
            Self::UsedIn => "USED_IN",
            Self::RelatedTo => "RELATED_TO",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SIMILAR" => Some(Self::Similar),
            "DERIVED_FROM" => Some(Self::DerivedFrom),
            "USED_IN" => Some(Self::UsedIn),
            "RELATED_TO" => Some(Self::RelatedTo),
            _ => None,
        }
    }
}

/// A relationship between two cognitive assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: RelationshipType,
    pub weight: f32,
}

impl Relationship {
    pub fn new(source_id: Uuid, target_id: Uuid, relationship_type: RelationshipType, weight: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id,
            target_id,
            relationship_type,
            weight,
        }
    }
}

/// A node in the relationship graph for Cytoscape.js.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "prompt", "workflow", "tag", "intent"
    pub metadata: serde_json::Value,
}

/// An edge in the relationship graph for Cytoscape.js.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub weight: f32,
}

/// Complete graph data for Cytoscape.js rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
