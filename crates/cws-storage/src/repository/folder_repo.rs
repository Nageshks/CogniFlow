use rusqlite::{params, Connection};
use uuid::Uuid;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::{Folder, FolderTreeNode, MAX_FOLDER_DEPTH};

/// Create a folder, validating depth limit.
pub fn create(conn: &Connection, name: String, parent_id: Option<Uuid>) -> CwsResult<Folder> {
    let depth = if let Some(pid) = parent_id {
        let parent_depth: i32 = conn
            .query_row(
                "SELECT depth FROM folders WHERE id = ?1",
                params![pid.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => CwsError::NotFound {
                    entity: "Folder".to_string(),
                    id: pid.to_string(),
                },
                _ => CwsError::Database(e.to_string()),
            })?;
        parent_depth + 1
    } else {
        0
    };

    if depth > MAX_FOLDER_DEPTH {
        return Err(CwsError::FolderDepthExceeded {
            max: MAX_FOLDER_DEPTH,
        });
    }

    let folder = Folder::new(name, parent_id, depth);

    conn.execute(
        "INSERT INTO folders (id, name, parent_id, depth) VALUES (?1, ?2, ?3, ?4)",
        params![
            folder.id.to_string(),
            folder.name,
            folder.parent_id.map(|p| p.to_string()),
            folder.depth,
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(folder)
}

/// List all folders (flat).
pub fn list(conn: &Connection) -> CwsResult<Vec<Folder>> {
    let mut stmt = conn
        .prepare("SELECT id, name, parent_id, depth FROM folders ORDER BY name ASC")
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Folder {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                parent_id: row
                    .get::<_, Option<String>>(2)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                depth: row.get(3)?,
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut folders = Vec::new();
    for row in rows {
        folders.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(folders)
}

/// Build a recursive folder tree.
pub fn get_tree(conn: &Connection) -> CwsResult<Vec<FolderTreeNode>> {
    let folders = list(conn)?;

    // Count prompts per folder
    let mut stmt = conn
        .prepare("SELECT folder_id, COUNT(*) FROM prompts WHERE folder_id IS NOT NULL GROUP BY folder_id")
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let counts: std::collections::HashMap<Uuid, i32> = stmt
        .query_map([], |row| {
            let fid_str: String = row.get(0)?;
            let count: i32 = row.get(1)?;
            Ok((Uuid::parse_str(&fid_str).unwrap(), count))
        })
        .map_err(|e| CwsError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    // Build tree recursively
    fn build_children(
        parent_id: Option<Uuid>,
        folders: &[Folder],
        counts: &std::collections::HashMap<Uuid, i32>,
    ) -> Vec<FolderTreeNode> {
        folders
            .iter()
            .filter(|f| f.parent_id == parent_id)
            .map(|f| {
                let children = build_children(Some(f.id), folders, counts);
                FolderTreeNode {
                    folder: f.clone(),
                    children,
                    prompt_count: *counts.get(&f.id).unwrap_or(&0),
                }
            })
            .collect()
    }

    Ok(build_children(None, &folders, &counts))
}

/// Rename a folder.
pub fn rename(conn: &Connection, id: Uuid, name: String) -> CwsResult<()> {
    let affected = conn
        .execute(
            "UPDATE folders SET name = ?1 WHERE id = ?2",
            params![name, id.to_string()],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;
    if affected == 0 {
        return Err(CwsError::NotFound {
            entity: "Folder".to_string(),
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Delete a folder. Children have parent_id set to NULL via FK cascade.
pub fn delete(conn: &Connection, id: Uuid) -> CwsResult<()> {
    // Update child folders to have no parent
    conn.execute(
        "UPDATE folders SET parent_id = NULL, depth = 0 WHERE parent_id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    // Clear folder_id on prompts in this folder
    conn.execute(
        "UPDATE prompts SET folder_id = NULL WHERE folder_id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    let affected = conn
        .execute("DELETE FROM folders WHERE id = ?1", params![id.to_string()])
        .map_err(|e| CwsError::Database(e.to_string()))?;

    if affected == 0 {
        return Err(CwsError::NotFound {
            entity: "Folder".to_string(),
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Move a prompt into a folder.
pub fn move_prompt_to_folder(
    conn: &Connection,
    prompt_id: Uuid,
    folder_id: Option<Uuid>,
) -> CwsResult<()> {
    conn.execute(
        "UPDATE prompts SET folder_id = ?1 WHERE id = ?2",
        params![
            folder_id.map(|f| f.to_string()),
            prompt_id.to_string(),
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;
    Ok(())
}
