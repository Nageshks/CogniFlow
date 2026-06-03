use rusqlite::Connection;
use cws_core::error::CwsError;
use std::path::Path;
use tracing::info;

use crate::migrations;

/// Main database wrapper around a rusqlite Connection.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Open (or create) the SQLite database at the given path.
    pub fn open(path: &Path) -> Result<Self, CwsError> {
        let connection = Connection::open(path)
            .map_err(|e| CwsError::Database(e.to_string()))?;

        // Enable WAL mode for better concurrent reads
        connection
            .execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| CwsError::Database(e.to_string()))?;

        let db = Self { connection };
        db.run_migrations()?;

        info!("Database opened at {:?}", path);
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, CwsError> {
        let connection = Connection::open_in_memory()
            .map_err(|e| CwsError::Database(e.to_string()))?;

        connection
            .execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| CwsError::Database(e.to_string()))?;

        let db = Self { connection };
        db.run_migrations()?;

        info!("In-memory database opened");
        Ok(db)
    }

    /// Run all migrations to create tables.
    pub fn run_migrations(&self) -> Result<(), CwsError> {
        self.connection
            .execute_batch(migrations::CREATE_TABLES)
            .map_err(|e| CwsError::Database(format!("Migration failed: {}", e)))?;
        let _ = self.connection.execute("ALTER TABLE workflows ADD COLUMN folder_id TEXT", []);
        info!("Database migrations applied successfully");
        Ok(())
    }

    /// Get a reference to the underlying connection.
    pub fn conn(&self) -> &Connection {
        &self.connection
    }
}
