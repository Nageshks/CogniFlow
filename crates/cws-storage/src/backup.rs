use std::fs;
use std::path::Path;

use chrono::Utc;
use tracing::info;

use cws_core::error::{CwsError, CwsResult};
use cws_core::models::DataStats;

use crate::db::Database;

/// Create a backup of the database file.
pub fn create_backup(db_path: &Path, backup_dir: &Path) -> CwsResult<()> {
    fs::create_dir_all(backup_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("cogniflow_backup_{}.db", timestamp);
    let backup_path = backup_dir.join(backup_name);

    fs::copy(db_path, &backup_path)
        .map_err(|e| CwsError::ImportExport(format!("Failed to create backup: {}", e)))?;

    info!("Backup created at {:?}", backup_path);
    Ok(())
}

/// Remove old backups, keeping only the most recent `keep_count`.
pub fn cleanup_old_backups(backup_dir: &Path, keep_count: usize) -> CwsResult<()> {
    if !backup_dir.exists() {
        return Ok(());
    }

    let mut backups: Vec<_> = fs::read_dir(backup_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|n| n.starts_with("cogniflow_backup_") && n.ends_with(".db"))
                .unwrap_or(false)
        })
        .collect();

    // Sort by name (which includes timestamp) in descending order
    backups.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    // Remove backups beyond keep_count
    for backup in backups.iter().skip(keep_count) {
        fs::remove_file(backup.path())?;
        info!("Removed old backup: {:?}", backup.path());
    }

    Ok(())
}

/// Get statistics about the data in the database.
pub fn get_stats(db: &Database) -> CwsResult<DataStats> {
    let conn = db.conn();

    let total_prompts: i64 = conn
        .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let total_workflows: i64 = conn
        .query_row("SELECT COUNT(*) FROM workflows", [], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let total_tags: i64 = conn
        .query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let total_folders: i64 = conn
        .query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let total_relationships: i64 = conn
        .query_row("SELECT COUNT(*) FROM relationships", [], |row| row.get(0))
        .map_err(|e| CwsError::Database(e.to_string()))?;

    let archived_prompts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM prompts WHERE archived = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| CwsError::Database(e.to_string()))?;

    Ok(DataStats {
        total_prompts,
        total_workflows,
        total_tags,
        total_folders,
        total_relationships,
        archived_prompts,
    })
}
