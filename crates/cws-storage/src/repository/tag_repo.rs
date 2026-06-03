use rusqlite::{params, Connection};
use uuid::Uuid;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::Tag;

/// Create a tag or return the existing one (upsert by name).
pub fn create_or_get(conn: &Connection, name: &str) -> CwsResult<Tag> {
    // Try to find existing tag
    let existing = conn.query_row(
        "SELECT id, name FROM tags WHERE name = ?1",
        params![name],
        |row| {
            Ok(Tag {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
            })
        },
    );

    match existing {
        Ok(tag) => Ok(tag),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let tag = Tag::new(name.to_string());
            conn.execute(
                "INSERT INTO tags (id, name) VALUES (?1, ?2)",
                params![tag.id.to_string(), tag.name],
            )
            .map_err(|e| CwsError::Database(e.to_string()))?;
            Ok(tag)
        }
        Err(e) => Err(CwsError::Database(e.to_string())),
    }
}

/// List all tags.
pub fn list(conn: &Connection) -> CwsResult<Vec<Tag>> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM tags ORDER BY name ASC")
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(tags)
}

/// Get tags for a specific prompt.
pub fn get_prompt_tags(conn: &Connection, prompt_id: Uuid) -> CwsResult<Vec<Tag>> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name FROM tags t
             INNER JOIN prompt_tags pt ON t.id = pt.tag_id
             WHERE pt.prompt_id = ?1
             ORDER BY t.name ASC",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![prompt_id.to_string()], |row| {
            Ok(Tag {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(tags)
}

/// Set tags for a prompt (replace all existing tags).
pub fn set_prompt_tags(
    conn: &Connection,
    prompt_id: Uuid,
    tag_names: Vec<String>,
) -> CwsResult<()> {
    // Remove all existing tag associations
    conn.execute(
        "DELETE FROM prompt_tags WHERE prompt_id = ?1",
        params![prompt_id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    // Create or get each tag and associate it
    for name in tag_names {
        let tag = create_or_get(conn, &name)?;
        conn.execute(
            "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
            params![prompt_id.to_string(), tag.id.to_string()],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;
    }

    Ok(())
}

/// Delete a tag by ID.
pub fn delete(conn: &Connection, id: Uuid) -> CwsResult<()> {
    // Remove from prompt_tags first
    conn.execute(
        "DELETE FROM prompt_tags WHERE tag_id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    conn.execute("DELETE FROM tags WHERE id = ?1", params![id.to_string()])
        .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(())
}
