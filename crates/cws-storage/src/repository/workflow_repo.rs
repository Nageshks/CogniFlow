use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::{
    CreateWorkflowRequest, CreateWorkflowStepRequest, ReorderStepsRequest,
    UpdateWorkflowRequest, UpdateWorkflowStepRequest, Workflow, WorkflowStep, WorkflowWithSteps,
};

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Create a new workflow.
pub fn create(conn: &Connection, req: &CreateWorkflowRequest) -> CwsResult<Workflow> {
    let desc = req.description.clone().unwrap_or_default();
    let intent = req.intent.clone().unwrap_or_default();
    let workflow = Workflow::new(req.title.clone(), desc, intent, req.folder_id);

    conn.execute(
        "INSERT INTO workflows (id, title, description, intent, folder_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            workflow.id.to_string(),
            workflow.title,
            workflow.description,
            workflow.intent,
            workflow.folder_id.map(|f| f.to_string()),
            workflow.created_at.to_rfc3339(),
            workflow.updated_at.to_rfc3339(),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(workflow)
}

/// Get a workflow by ID with all its steps.
pub fn get_by_id(conn: &Connection, id: Uuid) -> CwsResult<WorkflowWithSteps> {
    let workflow = conn
        .query_row(
            "SELECT id, title, description, intent, folder_id, created_at, updated_at
             FROM workflows WHERE id = ?1",
            params![id.to_string()],
            |row| {
                Ok(Workflow {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    intent: row.get(3)?,
                    folder_id: row.get::<_, Option<String>>(4)?.and_then(|s| Uuid::parse_str(&s).ok()),
                    created_at: parse_dt(&row.get::<_, String>(5)?),
                    updated_at: parse_dt(&row.get::<_, String>(6)?),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => CwsError::NotFound {
                entity: "Workflow".to_string(),
                id: id.to_string(),
            },
            _ => CwsError::Database(e.to_string()),
        })?;

    let steps = get_steps(conn, id)?;

    Ok(WorkflowWithSteps { workflow, steps })
}

/// List all workflows (without steps).
pub fn list(conn: &Connection) -> CwsResult<Vec<Workflow>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, intent, folder_id, created_at, updated_at
             FROM workflows ORDER BY updated_at DESC",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Workflow {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                title: row.get(1)?,
                description: row.get(2)?,
                intent: row.get(3)?,
                folder_id: row.get::<_, Option<String>>(4)?.and_then(|s| Uuid::parse_str(&s).ok()),
                created_at: parse_dt(&row.get::<_, String>(5)?),
                updated_at: parse_dt(&row.get::<_, String>(6)?),
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut workflows = Vec::new();
    for row in rows {
        workflows.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(workflows)
}

/// Update a workflow.
pub fn update(conn: &Connection, req: &UpdateWorkflowRequest) -> CwsResult<Workflow> {
    let existing = get_by_id(conn, req.id)?;
    let now = Utc::now();

    let new_title = req.title.as_ref().unwrap_or(&existing.workflow.title);
    let new_desc = req.description.as_ref().unwrap_or(&existing.workflow.description);
    let new_intent = req.intent.as_ref().unwrap_or(&existing.workflow.intent);
    let new_folder_id = match &req.folder_id {
        Some(fid) => *fid,
        None => existing.workflow.folder_id,
    };

    conn.execute(
        "UPDATE workflows SET title = ?1, description = ?2, intent = ?3, folder_id = ?4, updated_at = ?5 WHERE id = ?6",
        params![
            new_title,
            new_desc,
            new_intent,
            new_folder_id.map(|f| f.to_string()),
            now.to_rfc3339(),
            req.id.to_string(),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(Workflow {
        id: req.id,
        title: new_title.clone(),
        description: new_desc.clone(),
        intent: new_intent.clone(),
        folder_id: new_folder_id,
        created_at: existing.workflow.created_at,
        updated_at: now,
    })
}

/// Delete a workflow and all its steps (CASCADE handles steps).
pub fn delete(conn: &Connection, id: Uuid) -> CwsResult<()> {
    // Delete steps first (in case FK cascade not working)
    conn.execute(
        "DELETE FROM workflow_steps WHERE workflow_id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    let affected = conn
        .execute("DELETE FROM workflows WHERE id = ?1", params![id.to_string()])
        .map_err(|e| CwsError::Database(e.to_string()))?;

    if affected == 0 {
        return Err(CwsError::NotFound {
            entity: "Workflow".to_string(),
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Add a step to a workflow.
pub fn add_step(conn: &Connection, req: &CreateWorkflowStepRequest) -> CwsResult<WorkflowStep> {
    let step = WorkflowStep::new(
        req.workflow_id,
        req.order_index,
        req.title.clone(),
        req.instruction.clone(),
        req.prompt_id,
    );

    conn.execute(
        "INSERT INTO workflow_steps (id, workflow_id, order_index, title, instruction, prompt_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            step.id.to_string(),
            step.workflow_id.to_string(),
            step.order_index,
            step.title,
            step.instruction,
            step.prompt_id.map(|p| p.to_string()),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    // Touch the workflow's updated_at
    touch_workflow(conn, req.workflow_id)?;

    Ok(step)
}

/// Update a workflow step.
pub fn update_step(conn: &Connection, req: &UpdateWorkflowStepRequest) -> CwsResult<WorkflowStep> {
    let existing = get_step_by_id(conn, req.id)?;

    let new_title = req.title.as_ref().unwrap_or(&existing.title);
    let new_instruction = req.instruction.as_ref().unwrap_or(&existing.instruction);
    let new_prompt_id = match &req.prompt_id {
        Some(pid) => *pid,
        None => existing.prompt_id,
    };

    conn.execute(
        "UPDATE workflow_steps SET title = ?1, instruction = ?2, prompt_id = ?3 WHERE id = ?4",
        params![
            new_title,
            new_instruction,
            new_prompt_id.map(|p| p.to_string()),
            req.id.to_string(),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    touch_workflow(conn, existing.workflow_id)?;

    Ok(WorkflowStep {
        id: req.id,
        workflow_id: existing.workflow_id,
        order_index: existing.order_index,
        title: new_title.clone(),
        instruction: new_instruction.clone(),
        prompt_id: new_prompt_id,
    })
}

/// Delete a workflow step.
pub fn delete_step(conn: &Connection, id: Uuid) -> CwsResult<()> {
    let step = get_step_by_id(conn, id)?;
    conn.execute(
        "DELETE FROM workflow_steps WHERE id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    touch_workflow(conn, step.workflow_id)?;
    Ok(())
}

/// Reorder steps in a workflow.
pub fn reorder_steps(conn: &Connection, req: &ReorderStepsRequest) -> CwsResult<()> {
    for (step_id, new_order) in &req.order {
        conn.execute(
            "UPDATE workflow_steps SET order_index = ?1 WHERE id = ?2 AND workflow_id = ?3",
            params![new_order, step_id.to_string(), req.workflow_id.to_string()],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;
    }

    touch_workflow(conn, req.workflow_id)?;
    Ok(())
}

/// Link a prompt to a workflow step.
pub fn link_prompt_to_step(conn: &Connection, step_id: Uuid, prompt_id: Uuid) -> CwsResult<()> {
    let step = get_step_by_id(conn, step_id)?;
    conn.execute(
        "UPDATE workflow_steps SET prompt_id = ?1 WHERE id = ?2",
        params![prompt_id.to_string(), step_id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    touch_workflow(conn, step.workflow_id)?;
    Ok(())
}

// ─── helpers ────────────────────────────────────────────────────────────────

pub fn get_steps(conn: &Connection, workflow_id: Uuid) -> CwsResult<Vec<WorkflowStep>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, workflow_id, order_index, title, instruction, prompt_id
             FROM workflow_steps WHERE workflow_id = ?1 ORDER BY order_index ASC",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![workflow_id.to_string()], |row| {
            Ok(WorkflowStep {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                workflow_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                order_index: row.get(2)?,
                title: row.get(3)?,
                instruction: row.get(4)?,
                prompt_id: row
                    .get::<_, Option<String>>(5)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut steps = Vec::new();
    for row in rows {
        steps.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(steps)
}

fn get_step_by_id(conn: &Connection, id: Uuid) -> CwsResult<WorkflowStep> {
    conn.query_row(
        "SELECT id, workflow_id, order_index, title, instruction, prompt_id
         FROM workflow_steps WHERE id = ?1",
        params![id.to_string()],
        |row| {
            Ok(WorkflowStep {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                workflow_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                order_index: row.get(2)?,
                title: row.get(3)?,
                instruction: row.get(4)?,
                prompt_id: row
                    .get::<_, Option<String>>(5)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CwsError::NotFound {
            entity: "WorkflowStep".to_string(),
            id: id.to_string(),
        },
        _ => CwsError::Database(e.to_string()),
    })
}

fn touch_workflow(conn: &Connection, workflow_id: Uuid) -> CwsResult<()> {
    let now = Utc::now();
    conn.execute(
        "UPDATE workflows SET updated_at = ?1 WHERE id = ?2",
        params![now.to_rfc3339(), workflow_id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;
    Ok(())
}

pub fn get_workflows_using_prompt(conn: &Connection, prompt_id: Uuid) -> CwsResult<Vec<Workflow>> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT w.id, w.title, w.description, w.intent, w.folder_id, w.created_at, w.updated_at
             FROM workflows w
             JOIN workflow_steps s ON s.workflow_id = w.id
             WHERE s.prompt_id = ?1
             ORDER BY w.updated_at DESC",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![prompt_id.to_string()], |row| {
            Ok(Workflow {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                title: row.get(1)?,
                description: row.get(2)?,
                intent: row.get(3)?,
                folder_id: row.get::<_, Option<String>>(4)?.and_then(|s| Uuid::parse_str(&s).ok()),
                created_at: parse_dt(&row.get::<_, String>(5)?),
                updated_at: parse_dt(&row.get::<_, String>(6)?),
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut workflows = Vec::new();
    for row in rows {
        workflows.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(workflows)
}
