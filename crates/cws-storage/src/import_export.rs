use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::info;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::{CreatePromptRequest, Prompt, Workflow};

use crate::db::Database;
use crate::repository::{prompt_repo, workflow_repo, tag_repo, category_repo, folder_repo};

/// Complete export data structure.
#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    version: String,
    prompts: Vec<PromptExport>,
    workflows: Vec<WorkflowExport>,
    folders: Vec<cws_core::models::Folder>,
    tags: Vec<cws_core::models::Tag>,
    categories: Vec<cws_core::models::Category>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptExport {
    #[serde(flatten)]
    prompt: Prompt,
    tags: Vec<String>,
    categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowExport {
    #[serde(flatten)]
    workflow: cws_core::models::WorkflowWithSteps,
}

/// Export all data as JSON.
pub fn export_json(db: &Database, path: &Path) -> CwsResult<()> {
    let conn = db.conn();

    let prompts_raw = prompt_repo::list(conn, false, None)?;
    let archived = prompt_repo::list(conn, true, None)?;

    let mut prompts: Vec<PromptExport> = Vec::new();
    for pm in prompts_raw.into_iter().chain(archived.into_iter()) {
        prompts.push(PromptExport {
            prompt: pm.prompt,
            tags: pm.tags,
            categories: pm.categories,
        });
    }

    let workflow_list = workflow_repo::list(conn)?;
    let mut workflows = Vec::new();
    for w in workflow_list {
        let wws = workflow_repo::get_by_id(conn, w.id)?;
        workflows.push(WorkflowExport { workflow: wws });
    }

    let folders = folder_repo::list(conn)?;
    let tags = tag_repo::list(conn)?;
    let categories = category_repo::list(conn)?;

    let export = ExportData {
        version: "1.0".to_string(),
        prompts,
        workflows,
        folders,
        tags,
        categories,
    };

    let json = serde_json::to_string_pretty(&export)?;
    fs::write(path, json)?;

    info!("Exported data to {:?}", path);
    Ok(())
}

/// Export prompts and workflows as markdown files.
pub fn export_markdown(db: &Database, path: &Path, expanded: bool) -> CwsResult<()> {
    let conn = db.conn();
    fs::create_dir_all(path)?;

    // Export prompts
    let prompts_dir = path.join("prompts");
    fs::create_dir_all(&prompts_dir)?;

    let prompts = prompt_repo::list(conn, false, None)?;
    for pm in &prompts {
        let filename = sanitize_filename(&pm.prompt.title);
        let filepath = prompts_dir.join(format!("{}.md", filename));
        let mut content = format!("# {}\n\n", pm.prompt.title);
        if !pm.prompt.description.is_empty() {
            content.push_str(&format!("> {}\n\n", pm.prompt.description));
        }
        if !pm.tags.is_empty() {
            content.push_str(&format!("**Tags:** {}\n\n", pm.tags.join(", ")));
        }
        if !pm.categories.is_empty() {
            content.push_str(&format!("**Categories:** {}\n\n", pm.categories.join(", ")));
        }
        content.push_str("---\n\n");
        content.push_str(&pm.prompt.content);
        content.push('\n');
        fs::write(&filepath, content)?;
    }

    // Export workflows
    let workflows_dir = path.join("workflows");
    fs::create_dir_all(&workflows_dir)?;

    let workflows = workflow_repo::list(conn)?;
    for w in &workflows {
        let wws = workflow_repo::get_by_id(conn, w.id)?;
        let filename = sanitize_filename(&w.title);

        if expanded {
            let wf_dir = workflows_dir.join(&filename);
            fs::create_dir_all(&wf_dir)?;
            // Write workflow overview
            let overview = format!(
                "# {}\n\n> {}\n\n**Intent:** {}\n\n**Steps:** {}\n",
                w.title,
                w.description,
                w.intent,
                wws.steps.len()
            );
            fs::write(wf_dir.join("README.md"), overview)?;
            // Write each step
            for step in &wws.steps {
                let step_file = wf_dir.join(format!("{:02}_{}.md", step.order_index, sanitize_filename(&step.title)));
                let step_content = format!(
                    "# Step {}: {}\n\n{}\n",
                    step.order_index, step.title, step.instruction
                );
                fs::write(step_file, step_content)?;
            }
        } else {
            let filepath = workflows_dir.join(format!("{}.md", filename));
            let mut content = format!("# {}\n\n> {}\n\n**Intent:** {}\n\n---\n\n", w.title, w.description, w.intent);
            for step in &wws.steps {
                content.push_str(&format!("## Step {}: {}\n\n{}\n\n", step.order_index, step.title, step.instruction));
            }
            fs::write(&filepath, content)?;
        }
    }

    info!("Exported markdown to {:?}", path);
    Ok(())
}

/// Import data from a JSON file.
pub fn import_json(db: &Database, path: &Path) -> CwsResult<()> {
    let conn = db.conn();
    let content = fs::read_to_string(path)?;
    let data: ExportData = serde_json::from_str(&content)?;

    info!("Importing {} prompts, {} workflows", data.prompts.len(), data.workflows.len());

    for pe in &data.prompts {
        let req = CreatePromptRequest {
            title: pe.prompt.title.clone(),
            content: pe.prompt.content.clone(),
            description: pe.prompt.description.clone(),
            folder_id: pe.prompt.folder_id,
            tags: pe.tags.clone(),
            category: pe.categories.first().cloned(),
        };
        let created = prompt_repo::create(conn, &req)?;

        // Set tags
        if !pe.tags.is_empty() {
            tag_repo::set_prompt_tags(conn, created.id, pe.tags.clone())?;
        }
        // Set categories
        if !pe.categories.is_empty() {
            category_repo::set_prompt_categories(conn, created.id, pe.categories.clone())?;
        }
    }

    for we in &data.workflows {
        let wf_req = cws_core::models::CreateWorkflowRequest {
            title: we.workflow.workflow.title.clone(),
            description: Some(we.workflow.workflow.description.clone()),
            intent: Some(we.workflow.workflow.intent.clone()),
            folder_id: we.workflow.workflow.folder_id,
        };
        let created_wf = workflow_repo::create(conn, &wf_req)?;

        for step in &we.workflow.steps {
            let step_req = cws_core::models::CreateWorkflowStepRequest {
                workflow_id: created_wf.id,
                order_index: step.order_index,
                title: step.title.clone(),
                instruction: step.instruction.clone(),
                prompt_id: step.prompt_id,
            };
            workflow_repo::add_step(conn, &step_req)?;
        }
    }

    info!("Import complete");
    Ok(())
}

/// Import prompts from markdown files in a directory.
pub fn import_markdown(db: &Database, path: &Path) -> CwsResult<()> {
    let conn = db.conn();

    if path.is_file() {
        import_single_markdown(conn, path)?;
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().map(|e| e == "md").unwrap_or(false) {
                import_single_markdown(conn, &file_path)?;
            }
        }
    }

    info!("Markdown import complete from {:?}", path);
    Ok(())
}

/// Import text files as prompts.
pub fn import_txt(db: &Database, path: &Path) -> CwsResult<()> {
    let conn = db.conn();

    if path.is_file() {
        import_single_txt(conn, path)?;
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().map(|e| e == "txt").unwrap_or(false) {
                import_single_txt(conn, &file_path)?;
            }
        }
    }

    info!("Text import complete from {:?}", path);
    Ok(())
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn import_single_markdown(conn: &rusqlite::Connection, path: &Path) -> CwsResult<()> {
    let content = fs::read_to_string(path)?;
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Prompt")
        .to_string();

    // Try to extract title from # heading
    let (parsed_title, body) = if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("# ") {
            let t = first_line.trim_start_matches("# ").to_string();
            let b = content
                .lines()
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            (t, b)
        } else {
            (title, content)
        }
    } else {
        (title, content)
    };

    let req = CreatePromptRequest {
        title: parsed_title,
        content: body,
        description: String::new(),
        folder_id: None,
        tags: Vec::new(),
        category: None,
    };
    prompt_repo::create(conn, &req)?;
    Ok(())
}

fn import_single_txt(conn: &rusqlite::Connection, path: &Path) -> CwsResult<()> {
    let content = fs::read_to_string(path)?;
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Prompt")
        .to_string();

    let req = CreatePromptRequest {
        title,
        content,
        description: String::new(),
        folder_id: None,
        tags: Vec::new(),
        category: None,
    };
    prompt_repo::create(conn, &req)?;
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
}
