use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::{
    CreatePromptRequest, Prompt, PromptVersion, PromptWithMeta, UpdatePromptRequest,
};

/// Parse a DateTime<Utc> from an RFC 3339 string stored in SQLite.
fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Create a new prompt and its initial version.
pub fn create(conn: &Connection, req: &CreatePromptRequest) -> CwsResult<Prompt> {
    let prompt = Prompt::new(req.title.clone(), req.content.clone(), req.description.clone());
    let prompt_with_folder = Prompt {
        folder_id: req.folder_id,
        ..prompt
    };

    conn.execute(
        "INSERT INTO prompts (id, title, content, description, folder_id, created_at, updated_at, archived, current_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            prompt_with_folder.id.to_string(),
            prompt_with_folder.title,
            prompt_with_folder.content,
            prompt_with_folder.description,
            prompt_with_folder.folder_id.map(|f| f.to_string()),
            prompt_with_folder.created_at.to_rfc3339(),
            prompt_with_folder.updated_at.to_rfc3339(),
            prompt_with_folder.archived as i32,
            prompt_with_folder.current_version,
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    // Create initial version
    let version = PromptVersion::new(
        prompt_with_folder.id,
        1,
        prompt_with_folder.content.clone(),
        "Initial version".to_string(),
    );
    conn.execute(
        "INSERT INTO prompt_versions (id, prompt_id, version_number, content, change_summary, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            version.id.to_string(),
            version.prompt_id.to_string(),
            version.version_number,
            version.content,
            version.change_summary,
            version.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(prompt_with_folder)
}

/// Get a prompt by ID.
pub fn get_by_id(conn: &Connection, id: Uuid) -> CwsResult<Prompt> {
    conn.query_row(
        "SELECT id, title, content, description, folder_id, created_at, updated_at, archived, current_version
         FROM prompts WHERE id = ?1",
        params![id.to_string()],
        |row| {
            Ok(Prompt {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                title: row.get(1)?,
                content: row.get(2)?,
                description: row.get(3)?,
                folder_id: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                created_at: parse_dt(&row.get::<_, String>(5)?),
                updated_at: parse_dt(&row.get::<_, String>(6)?),
                archived: row.get::<_, i32>(7)? != 0,
                current_version: row.get(8)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CwsError::NotFound {
            entity: "Prompt".to_string(),
            id: id.to_string(),
        },
        _ => CwsError::Database(e.to_string()),
    })
}

/// List prompts, optionally filtering by archived status and folder.
pub fn list(
    conn: &Connection,
    archived: bool,
    folder_id: Option<Uuid>,
) -> CwsResult<Vec<PromptWithMeta>> {
    let mut sql = String::from(
        "SELECT id, title, content, description, folder_id, created_at, updated_at, archived, current_version
         FROM prompts WHERE archived = ?1",
    );
    let archived_val = archived as i32;

    if folder_id.is_some() {
        sql.push_str(" AND folder_id = ?2");
    }
    sql.push_str(" ORDER BY updated_at DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = if let Some(fid) = folder_id {
        stmt.query_map(params![archived_val, fid.to_string()], map_prompt_row)
    } else {
        stmt.query_map(params![archived_val], map_prompt_row)
    }
    .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        let prompt = row.map_err(|e| CwsError::Database(e.to_string()))?;
        let tags = get_prompt_tag_names(conn, prompt.id)?;
        let categories = get_prompt_category_names(conn, prompt.id)?;
        results.push(PromptWithMeta {
            prompt,
            tags,
            categories,
        });
    }

    Ok(results)
}

/// Update a prompt and create a new version if content changed.
pub fn update(conn: &Connection, req: &UpdatePromptRequest) -> CwsResult<Prompt> {
    let existing = get_by_id(conn, req.id)?;
    let now = Utc::now();

    let new_title = req.title.as_ref().unwrap_or(&existing.title);
    let new_content = req.content.as_ref().unwrap_or(&existing.content);
    let new_description = req.description.as_ref().unwrap_or(&existing.description);
    let new_folder_id = match &req.folder_id {
        Some(fid) => *fid,
        None => existing.folder_id,
    };

    let content_changed = new_content != &existing.content;
    let new_version = if content_changed {
        existing.current_version + 1
    } else {
        existing.current_version
    };

    conn.execute(
        "UPDATE prompts SET title = ?1, content = ?2, description = ?3, folder_id = ?4, updated_at = ?5, current_version = ?6
         WHERE id = ?7",
        params![
            new_title,
            new_content,
            new_description,
            new_folder_id.map(|f| f.to_string()),
            now.to_rfc3339(),
            new_version,
            req.id.to_string(),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    // Create a new version if content changed
    if content_changed {
        let version = PromptVersion::new(
            req.id,
            new_version,
            new_content.clone(),
            req.change_summary.clone(),
        );
        conn.execute(
            "INSERT INTO prompt_versions (id, prompt_id, version_number, content, change_summary, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                version.id.to_string(),
                version.prompt_id.to_string(),
                version.version_number,
                version.content,
                version.change_summary,
                version.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;
    }

    get_by_id(conn, req.id)
}

/// Soft-delete a prompt (set archived = true).
pub fn soft_delete(conn: &Connection, id: Uuid) -> CwsResult<()> {
    let affected = conn
        .execute(
            "UPDATE prompts SET archived = 1 WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;
    if affected == 0 {
        return Err(CwsError::NotFound {
            entity: "Prompt".to_string(),
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Toggle archived status.
pub fn archive(conn: &Connection, id: Uuid) -> CwsResult<()> {
    let prompt = get_by_id(conn, id)?;
    let new_archived = !prompt.archived;
    conn.execute(
        "UPDATE prompts SET archived = ?1 WHERE id = ?2",
        params![new_archived as i32, id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;
    Ok(())
}

/// Get all versions of a prompt.
pub fn get_versions(conn: &Connection, prompt_id: Uuid) -> CwsResult<Vec<PromptVersion>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, prompt_id, version_number, content, change_summary, created_at
             FROM prompt_versions WHERE prompt_id = ?1 ORDER BY version_number DESC",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![prompt_id.to_string()], |row| {
            Ok(PromptVersion {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                prompt_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                version_number: row.get(2)?,
                content: row.get(3)?,
                change_summary: row.get(4)?,
                created_at: parse_dt(&row.get::<_, String>(5)?),
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut versions = Vec::new();
    for row in rows {
        versions.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(versions)
}

/// Restore prompt content from a specific version.
pub fn restore_version(
    conn: &Connection,
    prompt_id: Uuid,
    version_number: i32,
) -> CwsResult<Prompt> {
    // Find the version
    let version_content: String = conn
        .query_row(
            "SELECT content FROM prompt_versions WHERE prompt_id = ?1 AND version_number = ?2",
            params![prompt_id.to_string(), version_number],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => CwsError::NotFound {
                entity: "PromptVersion".to_string(),
                id: format!("{}v{}", prompt_id, version_number),
            },
            _ => CwsError::Database(e.to_string()),
        })?;

    // Update prompt with restored content as a new version
    let req = UpdatePromptRequest {
        id: prompt_id,
        title: None,
        content: Some(version_content),
        description: None,
        folder_id: None,
        change_summary: format!("Restored from version {}", version_number),
    };

    update(conn, &req)
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn map_prompt_row(row: &rusqlite::Row) -> rusqlite::Result<Prompt> {
    Ok(Prompt {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
        title: row.get(1)?,
        content: row.get(2)?,
        description: row.get(3)?,
        folder_id: row
            .get::<_, Option<String>>(4)?
            .and_then(|s| Uuid::parse_str(&s).ok()),
        created_at: parse_dt(&row.get::<_, String>(5)?),
        updated_at: parse_dt(&row.get::<_, String>(6)?),
        archived: row.get::<_, i32>(7)? != 0,
        current_version: row.get(8)?,
    })
}

fn get_prompt_tag_names(conn: &Connection, prompt_id: Uuid) -> CwsResult<Vec<String>> {
    let mut stmt = conn
        .prepare(
            "SELECT t.name FROM tags t
             INNER JOIN prompt_tags pt ON t.id = pt.tag_id
             WHERE pt.prompt_id = ?1",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![prompt_id.to_string()], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut names = Vec::new();
    for row in rows {
        names.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(names)
}

fn get_prompt_category_names(conn: &Connection, prompt_id: Uuid) -> CwsResult<Vec<String>> {
    let mut stmt = conn
        .prepare(
            "SELECT c.name FROM categories c
             INNER JOIN prompt_categories pc ON c.id = pc.category_id
             WHERE pc.prompt_id = ?1",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![prompt_id.to_string()], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut names = Vec::new();
    for row in rows {
        names.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(names)
}
