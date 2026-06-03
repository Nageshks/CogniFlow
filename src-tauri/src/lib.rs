use std::sync::Mutex;
use std::path::Path;
use uuid::Uuid;
use tauri::{AppHandle, Manager, State};

use cws_core::error::CwsResult;
use cws_core::traits::{FullTextSearch, EmbeddingEngine};
use cws_core::models::{
    Prompt, PromptVersion, PromptWithMeta, CreatePromptRequest, UpdatePromptRequest,
    Workflow, WorkflowStep, WorkflowWithSteps, CreateWorkflowRequest, UpdateWorkflowRequest,
    CreateWorkflowStepRequest, UpdateWorkflowStepRequest, ReorderStepsRequest,
    SearchResult, HybridSearchConfig, AiSuggestions, IntentClassification,
    GraphData, Relationship, RelationshipType, Folder, FolderTreeNode,
    ExportRequest, ImportRequest, DataStats, ExportFormat, ImportFormat,
};

use cws_storage::repository::{
    prompt_repo, workflow_repo, folder_repo, tag_repo, category_repo, relationship_repo,
};

pub struct AppState {
    pub db: Mutex<cws_storage::Database>,
    pub search: Mutex<cws_search::SearchEngine>,
    pub ai: cws_ai::AiEngine,
    pub graph: cws_graph::GraphEngine,
}

// Helper to map CwsError to Tauri String errors
fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ===== PROMPT COMMANDS =====

#[tauri::command]
async fn create_prompt(
    state: State<'_, AppState>,
    request: CreatePromptRequest,
) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(map_err)?;
    let search = state.search.lock().map_err(map_err)?;
    let conn = db.conn();

    // Start a transaction so all DB updates succeed or fail together
    conn.execute("BEGIN TRANSACTION", []).map_err(map_err)?;

    let prompt_res = (|| -> CwsResult<Prompt> {
        let prompt = prompt_repo::create(conn, &request)?;
        
        // Save tags
        tag_repo::set_prompt_tags(conn, prompt.id, request.tags.clone())?;
        
        // Save category
        if let Some(cat) = &request.category {
            category_repo::set_prompt_categories(conn, prompt.id, vec![cat.clone()])?;
        }
        
        Ok(prompt)
    })();

    match prompt_res {
        Ok(prompt) => {
            conn.execute("COMMIT", []).map_err(map_err)?;
            
            // Index in Tantivy search
            let _ = search.index_document(
                &prompt.id.to_string(),
                "prompt",
                &prompt.title,
                &prompt.content,
                &prompt.description,
            );
            
            // Index in AI semantic search
            let _ = state.ai.index_entity(&prompt.id.to_string(), "prompt", &prompt.content);
            
            Ok(prompt)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_prompt(
    state: State<'_, AppState>,
    id: String,
) -> Result<PromptWithMeta, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;
    
    let prompt = prompt_repo::get_by_id(conn, uuid).map_err(map_err)?;
    let tags = tag_repo::get_prompt_tags(conn, uuid)
        .map_err(map_err)?
        .into_iter()
        .map(|t| t.name)
        .collect();
    let categories = category_repo::get_prompt_categories(conn, uuid)
        .map_err(map_err)?
        .into_iter()
        .map(|c| c.name)
        .collect();

    Ok(PromptWithMeta {
        prompt,
        tags,
        categories,
    })
}

#[tauri::command]
async fn list_prompts(
    state: State<'_, AppState>,
    folder_id: Option<String>,
    archived: Option<bool>,
) -> Result<Vec<PromptWithMeta>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    
    let fid = folder_id
        .and_then(|id| Uuid::parse_str(&id).ok());
    let is_archived = archived.unwrap_or(false);

    prompt_repo::list(conn, is_archived, fid).map_err(map_err)
}

#[tauri::command]
async fn update_prompt(
    state: State<'_, AppState>,
    request: UpdatePromptRequest,
) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(map_err)?;
    let search = state.search.lock().map_err(map_err)?;
    let conn = db.conn();

    let prompt = prompt_repo::update(conn, &request).map_err(map_err)?;
    
    // Update search index
    let _ = search.index_document(
        &prompt.id.to_string(),
        "prompt",
        &prompt.title,
        &prompt.content,
        &prompt.description,
    );
    
    // Update semantic AI vector store
    let _ = state.ai.index_entity(&prompt.id.to_string(), "prompt", &prompt.content);

    Ok(prompt)
}

#[tauri::command]
async fn delete_prompt(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let search = state.search.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    prompt_repo::soft_delete(conn, uuid).map_err(map_err)?;
    
    // Remove from indices
    let _ = search.remove_document(&id);
    let _ = state.ai.remove_entity(&id);

    Ok(())
}

#[tauri::command]
async fn archive_prompt(
    state: State<'_, AppState>,
    id: String,
    _archive: bool,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    // Just toggle archived flag
    prompt_repo::archive(conn, uuid).map_err(map_err)?;
    Ok(())
}

#[tauri::command]
async fn get_prompt_versions(
    state: State<'_, AppState>,
    prompt_id: String,
) -> Result<Vec<PromptVersion>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&prompt_id).map_err(map_err)?;

    prompt_repo::get_versions(conn, uuid).map_err(map_err)
}

#[tauri::command]
async fn restore_prompt_version(
    state: State<'_, AppState>,
    prompt_id: String,
    version_number: i32,
) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(map_err)?;
    let search = state.search.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&prompt_id).map_err(map_err)?;

    let restored = prompt_repo::restore_version(conn, uuid, version_number).map_err(map_err)?;
    
    // Reindex restored
    let _ = search.index_document(
        &restored.id.to_string(),
        "prompt",
        &restored.title,
        &restored.content,
        &restored.description,
    );
    let _ = state.ai.index_entity(&restored.id.to_string(), "prompt", &restored.content);

    Ok(restored)
}

// ===== WORKFLOW COMMANDS =====

#[tauri::command]
async fn create_workflow(
    state: State<'_, AppState>,
    request: CreateWorkflowRequest,
) -> Result<Workflow, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    let wf = workflow_repo::create(conn, &request).map_err(map_err)?;
    
    let search = state.search.lock().map_err(map_err)?;
    let _ = search.index_document(
        &wf.id.to_string(),
        "workflow",
        &wf.title,
        &wf.description,
        &wf.intent,
    );

    Ok(wf)
}

#[tauri::command]
async fn get_workflow(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkflowWithSteps, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    workflow_repo::get_by_id(conn, uuid).map_err(map_err)
}

#[tauri::command]
async fn list_workflows(
    state: State<'_, AppState>,
) -> Result<Vec<WorkflowWithSteps>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    let workflows = workflow_repo::list(conn).map_err(map_err)?;
    let mut w_with_steps = Vec::new();
    for w in workflows {
        let steps = workflow_repo::get_steps(conn, w.id).map_err(map_err)?;
        w_with_steps.push(WorkflowWithSteps { workflow: w, steps });
    }
    Ok(w_with_steps)
}

#[tauri::command]
async fn update_workflow(
    state: State<'_, AppState>,
    request: UpdateWorkflowRequest,
) -> Result<Workflow, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    let wf = workflow_repo::update(conn, &request).map_err(map_err)?;
    
    let search = state.search.lock().map_err(map_err)?;
    let _ = search.index_document(
        &wf.id.to_string(),
        "workflow",
        &wf.title,
        &wf.description,
        &wf.intent,
    );

    Ok(wf)
}

#[tauri::command]
async fn delete_workflow(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    workflow_repo::delete(conn, uuid).map_err(map_err)?;
    
    let search = state.search.lock().map_err(map_err)?;
    let _ = search.remove_document(&id);

    Ok(())
}

#[tauri::command]
async fn add_workflow_step(
    state: State<'_, AppState>,
    request: CreateWorkflowStepRequest,
) -> Result<WorkflowStep, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    workflow_repo::add_step(conn, &request).map_err(map_err)
}

#[tauri::command]
async fn update_workflow_step(
    state: State<'_, AppState>,
    request: UpdateWorkflowStepRequest,
) -> Result<WorkflowStep, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    workflow_repo::update_step(conn, &request).map_err(map_err)
}

#[tauri::command]
async fn delete_workflow_step(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    workflow_repo::delete_step(conn, uuid).map_err(map_err)
}

#[tauri::command]
async fn reorder_workflow_steps(
    state: State<'_, AppState>,
    request: ReorderStepsRequest,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    workflow_repo::reorder_steps(conn, &request).map_err(map_err)
}

#[tauri::command]
async fn link_prompt_to_step(
    state: State<'_, AppState>,
    step_id: String,
    prompt_id: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let sid = Uuid::parse_str(&step_id).map_err(map_err)?;
    let pid = match prompt_id {
        Some(id) => Uuid::parse_str(&id).map_err(map_err)?,
        None => return Err("Prompt ID is required".to_string()),
    };

    workflow_repo::link_prompt_to_step(conn, sid, pid).map_err(map_err)
}

#[tauri::command]
async fn get_workflows_using_prompt(
    state: State<'_, AppState>,
    prompt_id: String,
) -> Result<Vec<Workflow>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let pid = Uuid::parse_str(&prompt_id).map_err(map_err)?;

    workflow_repo::get_workflows_using_prompt(conn, pid).map_err(map_err)
}

// ===== SEARCH COMMANDS =====

#[tauri::command]
async fn search_fulltext(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let search = state.search.lock().map_err(map_err)?;
    cws_core::traits::FullTextSearch::search(&*search, &query, limit).map_err(map_err)
}

#[tauri::command]
async fn search_semantic(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    state.ai.semantic_search(&query, limit).map_err(map_err)
}

#[tauri::command]
async fn search_hybrid(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
    config: Option<HybridSearchConfig>,
) -> Result<Vec<SearchResult>, String> {
    let conf = config.unwrap_or_default();
    
    let ft_results = {
        let search = state.search.lock().map_err(map_err)?;
        cws_core::traits::FullTextSearch::search(&*search, &query, limit).unwrap_or_default()
    };
    
    let sem_results = state.ai.semantic_search(&query, limit).unwrap_or_default();

    // Blend hybrid results
    let mut blended: std::collections::HashMap<String, SearchResult> = std::collections::HashMap::new();

    for mut res in ft_results {
        res.score *= conf.keyword_weight;
        blended.insert(res.id.clone(), res);
    }

    for res in sem_results {
        if let Some(existing) = blended.get_mut(&res.id) {
            existing.score += res.score * conf.semantic_weight;
        } else {
            let mut weighted = res.clone();
            weighted.score *= conf.semantic_weight;
            blended.insert(weighted.id.clone(), weighted);
        }
    }

    let mut final_results: Vec<SearchResult> = blended.into_values().collect();
    final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    final_results.truncate(limit);

    Ok(final_results)
}

// ===== AI COMMANDS =====

#[tauri::command]
async fn suggest_tags(
    _state: State<'_, AppState>,
    content: String,
) -> Result<Vec<String>, String> {
    Ok(cws_ai::classifier::suggest_tags(&content))
}

#[tauri::command]
async fn suggest_category(
    _state: State<'_, AppState>,
    content: String,
) -> Result<Option<String>, String> {
    Ok(cws_ai::classifier::suggest_category(&content))
}

#[tauri::command]
async fn suggest_intent(
    _state: State<'_, AppState>,
    content: String,
) -> Result<AiSuggestions, String> {
    let tags = cws_ai::classifier::suggest_tags(&content);
    let category = cws_ai::classifier::suggest_category(&content);
    let intent = Some(cws_ai::classifier::classify_intent(&content));
    
    Ok(AiSuggestions {
        tags,
        category,
        intent,
    })
}

#[tauri::command]
async fn get_similar_assets(
    state: State<'_, AppState>,
    id: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    state.ai.semantic_search(&id, limit).map_err(map_err)
}

// ===== GRAPH COMMANDS =====

#[tauri::command]
async fn get_graph_data(
    state: State<'_, AppState>,
) -> Result<GraphData, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    // Fetch all prompts, workflows, and relationships
    let prompts_meta = prompt_repo::list(conn, false, None).map_err(map_err)?;
    
    let workflows_flat = workflow_repo::list(conn).map_err(map_err)?;
    let mut workflows = Vec::new();
    for w in workflows_flat {
        let w_with_steps = workflow_repo::get_by_id(conn, w.id).map_err(map_err)?;
        workflows.push(w_with_steps);
    }
    
    let relationships = relationship_repo::get_all(conn).map_err(map_err)?;

    let gd = state.graph.build_cytoscape_graph(&prompts_meta, &workflows, &relationships);
    Ok(gd)
}

#[tauri::command]
async fn get_node_relationships(
    state: State<'_, AppState>,
    node_id: String,
) -> Result<Vec<Relationship>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&node_id).map_err(map_err)?;

    relationship_repo::get_for_entity(conn, uuid).map_err(map_err)
}

#[tauri::command]
async fn add_relationship(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
    relationship_type: RelationshipType,
    weight: f32,
) -> Result<Relationship, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    
    let sid = Uuid::parse_str(&source_id).map_err(map_err)?;
    let tid = Uuid::parse_str(&target_id).map_err(map_err)?;
    
    let rel = Relationship::new(sid, tid, relationship_type, weight);
    relationship_repo::create(conn, &rel).map_err(map_err)
}

#[tauri::command]
async fn remove_relationship(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    relationship_repo::delete(conn, uuid).map_err(map_err)
}

// ===== FOLDER COMMANDS =====

#[tauri::command]
async fn create_folder(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<Folder, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    
    let pid = parent_id
        .and_then(|id| Uuid::parse_str(&id).ok());

    folder_repo::create(conn, name, pid).map_err(map_err)
}

#[tauri::command]
async fn rename_folder(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Folder, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    folder_repo::rename(conn, uuid, name.clone()).map_err(map_err)?;
    
    // Fetch updated folder depth/info
    let folders = folder_repo::list(conn).map_err(map_err)?;
    let folder = folders.iter().find(|f| f.id == uuid).ok_or("Folder not found after rename")?;
    Ok(folder.clone())
}

#[tauri::command]
async fn delete_folder_cmd(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    let uuid = Uuid::parse_str(&id).map_err(map_err)?;

    folder_repo::delete(conn, uuid).map_err(map_err)
}

#[tauri::command]
async fn move_to_folder_cmd(
    state: State<'_, AppState>,
    prompt_id: String,
    folder_id: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();
    
    let pid = Uuid::parse_str(&prompt_id).map_err(map_err)?;
    let fid = folder_id
        .and_then(|id| Uuid::parse_str(&id).ok());

    folder_repo::move_prompt_to_folder(conn, pid, fid).map_err(map_err)
}

#[tauri::command]
async fn list_folders_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<Folder>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    folder_repo::list(conn).map_err(map_err)
}

#[tauri::command]
async fn get_folder_tree_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<FolderTreeNode>, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    folder_repo::get_tree(conn).map_err(map_err)
}

// ===== IMPORT / EXPORT COMMANDS =====

#[tauri::command]
async fn export_data_cmd(
    state: State<'_, AppState>,
    request: ExportRequest,
) -> Result<String, String> {
    let db = state.db.lock().map_err(map_err)?;
    let path = Path::new(&request.path);

    match request.format {
        ExportFormat::Json => {
            cws_storage::import_export::export_json(&db, path).map_err(map_err)?;
        }
        ExportFormat::Markdown => {
            cws_storage::import_export::export_markdown(&db, path, request.expanded).map_err(map_err)?;
        }
    }
    
    Ok("Export completed successfully".to_string())
}

#[tauri::command]
async fn import_data_cmd(
    state: State<'_, AppState>,
    request: ImportRequest,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(map_err)?;
    let path = Path::new(&request.path);

    // Get count of prompts before import
    let count_before = {
        let conn = db.conn();
        let c: i64 = conn.query_row("SELECT COUNT(*) FROM prompts", [], |r| r.get(0)).unwrap_or(0);
        c as usize
    };

    match request.format {
        ImportFormat::Json => {
            cws_storage::import_export::import_json(&db, path).map_err(map_err)?;
        }
        ImportFormat::Markdown => {
            cws_storage::import_export::import_markdown(&db, path).map_err(map_err)?;
        }
        ImportFormat::Txt => {
            cws_storage::import_export::import_txt(&db, path).map_err(map_err)?;
        }
    }

    // Get count of prompts after import
    let count_after = {
        let conn = db.conn();
        let c: i64 = conn.query_row("SELECT COUNT(*) FROM prompts", [], |r| r.get(0)).unwrap_or(0);
        c as usize
    };

    let imported = count_after.saturating_sub(count_before);

    // Re-index all prompts to search engine
    let search = state.search.lock().map_err(map_err)?;
    let conn = db.conn();
    let prompts_meta = prompt_repo::list(conn, false, None).unwrap_or_default();
    for pm in prompts_meta {
        let _ = search.index_document(
            &pm.prompt.id.to_string(),
            "prompt",
            &pm.prompt.title,
            &pm.prompt.content,
            &pm.prompt.description,
        );
        let _ = state.ai.index_entity(&pm.prompt.id.to_string(), "prompt", &pm.prompt.content);
    }

    Ok(imported)
}

// ===== STATS COMMANDS =====

#[tauri::command]
async fn get_stats_cmd(
    state: State<'_, AppState>,
) -> Result<DataStats, String> {
    let db = state.db.lock().map_err(map_err)?;
    let conn = db.conn();

    let total_prompts: i64 = conn.query_row("SELECT COUNT(*) FROM prompts", [], |r| r.get(0)).map_err(map_err)?;
    let total_workflows: i64 = conn.query_row("SELECT COUNT(*) FROM workflows", [], |r| r.get(0)).map_err(map_err)?;
    let total_tags: i64 = conn.query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0)).map_err(map_err)?;
    let total_folders: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |r| r.get(0)).map_err(map_err)?;
    let total_relationships: i64 = conn.query_row("SELECT COUNT(*) FROM relationships", [], |r| r.get(0)).map_err(map_err)?;
    let archived_prompts: i64 = conn.query_row("SELECT COUNT(*) FROM prompts WHERE archived = 1", [], |r| r.get(0)).map_err(map_err)?;

    Ok(DataStats {
        total_prompts,
        total_workflows,
        total_tags,
        total_folders,
        total_relationships,
        archived_prompts,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle();
            let app_dir = app_handle.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
            std::fs::create_dir_all(&app_dir).ok();
            
            let db_path = app_dir.join("cogniflow.db");
            let index_path = app_dir.join("search_index");
            
            let db = cws_storage::Database::open(&db_path).expect("Failed to open database");
            let search = cws_search::SearchEngine::new(&index_path).expect("Failed to open search index");
            let ai = cws_ai::AiEngine::new().expect("Failed to open AI engine");
            let graph = cws_graph::GraphEngine::new();
            
            app.manage(AppState {
                db: Mutex::new(db),
                search: Mutex::new(search),
                ai,
                graph,
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_prompt,
            get_prompt,
            list_prompts,
            update_prompt,
            delete_prompt,
            archive_prompt,
            get_prompt_versions,
            restore_prompt_version,
            create_workflow,
            get_workflow,
            list_workflows,
            update_workflow,
            delete_workflow,
            add_workflow_step,
            update_workflow_step,
            delete_workflow_step,
            reorder_workflow_steps,
            link_prompt_to_step,
            get_workflows_using_prompt,
            search_fulltext,
            search_semantic,
            search_hybrid,
            suggest_tags,
            suggest_category,
            suggest_intent,
            get_similar_assets,
            get_graph_data,
            get_node_relationships,
            add_relationship,
            remove_relationship,
            create_folder,
            rename_folder,
            delete_folder_cmd,
            move_to_folder_cmd,
            list_folders_cmd,
            get_folder_tree_cmd,
            export_data_cmd,
            import_data_cmd,
            get_stats_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
