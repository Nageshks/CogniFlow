use rusqlite::{params, Connection};
use uuid::Uuid;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::{Relationship, RelationshipType};

/// Create a new relationship.
pub fn create(conn: &Connection, rel: &Relationship) -> CwsResult<Relationship> {
    conn.execute(
        "INSERT INTO relationships (id, source_id, target_id, relationship_type, weight)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            rel.id.to_string(),
            rel.source_id.to_string(),
            rel.target_id.to_string(),
            rel.relationship_type.as_str(),
            rel.weight,
        ],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(rel.clone())
}

/// Get all relationships involving a specific entity (as source or target).
pub fn get_for_entity(conn: &Connection, entity_id: Uuid) -> CwsResult<Vec<Relationship>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, source_id, target_id, relationship_type, weight
             FROM relationships
             WHERE source_id = ?1 OR target_id = ?1",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![entity_id.to_string()], map_relationship_row)
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(results)
}

/// Get all relationships.
pub fn get_all(conn: &Connection) -> CwsResult<Vec<Relationship>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, source_id, target_id, relationship_type, weight
             FROM relationships",
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], map_relationship_row)
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| CwsError::Database(e.to_string()))?);
    }
    Ok(results)
}

/// Delete a relationship by ID.
pub fn delete(conn: &Connection, id: Uuid) -> CwsResult<()> {
    let affected = conn
        .execute(
            "DELETE FROM relationships WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    if affected == 0 {
        return Err(CwsError::NotFound {
            entity: "Relationship".to_string(),
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Delete a relationship by source and target entity IDs.
pub fn delete_by_entities(conn: &Connection, source_id: Uuid, target_id: Uuid) -> CwsResult<()> {
    conn.execute(
        "DELETE FROM relationships WHERE
         (source_id = ?1 AND target_id = ?2) OR (source_id = ?2 AND target_id = ?1)",
        params![source_id.to_string(), target_id.to_string()],
    )
    .map_err(|e| CwsError::Database(e.to_string()))?;
    Ok(())
}

fn map_relationship_row(row: &rusqlite::Row) -> rusqlite::Result<Relationship> {
    let rel_type_str: String = row.get(3)?;
    Ok(Relationship {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
        source_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
        target_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap(),
        relationship_type: RelationshipType::from_str(&rel_type_str)
            .unwrap_or(RelationshipType::RelatedTo),
        weight: row.get(4)?,
    })
}
