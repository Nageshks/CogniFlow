use rusqlite::{params, Connection};
use uuid::Uuid;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::Category;

/// Create a category or return the existing one (upsert by name).
pub fn create_or_get(conn: &Connection, name: &str) -> CwsResult<Category> {
    let existing = conn.query_row(
        "SELECT id, name FROM categories WHERE name = ?1",
        params![name],
        |row| {
            Ok(Category {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
            })
        },
    );

    match existing {
        Ok(category) => Ok(category),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let category = Category::new(name.to_string());
            conn.execute(
                "INSERT INTO categories (id, name) VALUES (?1, ?2)",
                params![category.id.to_string(), category.name],
            )
            .map_err(|e| CwsError::Database(e.to_string()))?;
            Ok(category)
        }
        Err(e) => Err(CwsError::Database(e.to_string())),
    }
}

/// List all categories.
pub fn list(conn: &Connection) -> CwsResult<Vec<Category>> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM categories ORDER BY name ASC")
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Category {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut categories = Vec::new();
    for row in rows {
        categories.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(categories)
}

/// Get categories for a specific prompt.
pub fn get_prompt_categories(conn: &Connection, prompt_id: Uuid) -> CwsResult<Vec<Category>> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name FROM categories c
             INNER JOIN prompt_categories pc ON c.id = pc.category_id
             WHERE pc.prompt_id = ?1
             ORDER BY c.name ASC",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![prompt_id.to_string()], |row| {
            Ok(Category {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
            })
        })
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut categories = Vec::new();
    for row in rows {
        categories.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(categories)
}

/// Set categories for a prompt (replace all existing categories).
pub fn set_prompt_categories(
    conn: &Connection,
    prompt_id: Uuid,
    category_names: Vec<String>,
) -> CwsResult<()> {
    // Remove all existing category associations
    conn.execute(
        "DELETE FROM prompt_categories WHERE prompt_id = ?1",
        params![prompt_id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    // Create or get each category and associate it
    for name in category_names {
        let category = create_or_get(conn, &name)?;
        conn.execute(
            "INSERT OR IGNORE INTO prompt_categories (prompt_id, category_id) VALUES (?1, ?2)",
            params![prompt_id.to_string(), category.id.to_string()],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;
    }

    Ok(())
}

/// Delete a category by ID.
pub fn delete(conn: &Connection, id: Uuid) -> CwsResult<()> {
    conn.execute(
        "DELETE FROM prompt_categories WHERE category_id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    conn.execute(
        "DELETE FROM categories WHERE id = ?1",
        params![id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(())
}
