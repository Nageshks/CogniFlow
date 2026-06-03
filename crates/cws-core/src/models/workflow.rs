use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A cognitive workflow — an ordered sequence of steps that may reference prompts.
/// Internally stored as a DAG-ready structure for future branching support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub intent: String,
    pub folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    pub fn new(title: String, description: String, intent: String, folder_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            intent,
            folder_id,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A single step/node in a workflow. DAG-ready: uses node_id for future edge support
/// but currently uses order_index for linear ordering in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub order_index: i32,
    pub title: String,
    pub instruction: String,
    pub prompt_id: Option<Uuid>,
}

impl WorkflowStep {
    pub fn new(
        workflow_id: Uuid,
        order_index: i32,
        title: String,
        instruction: String,
        prompt_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            order_index,
            title,
            instruction,
            prompt_id,
        }
    }
}

/// A workflow with all its steps loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowWithSteps {
    #[serde(flatten)]
    pub workflow: Workflow,
    pub steps: Vec<WorkflowStep>,
}

/// Request to create a new workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub title: String,
    pub description: Option<String>,
    pub intent: Option<String>,
    pub folder_id: Option<Uuid>,
}

/// Request to update a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub intent: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
}

/// Request to create a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowStepRequest {
    pub workflow_id: Uuid,
    pub order_index: i32,
    pub title: String,
    pub instruction: String,
    pub prompt_id: Option<Uuid>,
}

/// Request to update a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowStepRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub instruction: Option<String>,
    pub prompt_id: Option<Option<Uuid>>,
}

/// Request to reorder steps in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderStepsRequest {
    pub workflow_id: Uuid,
    /// Vec of (step_id, new_order_index)
    pub order: Vec<(Uuid, i32)>,
}
