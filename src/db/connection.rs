use rusqlite::Connection;
use crate::error::Result;
use crate::config::Config;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn connect(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// Get a database connection, running migrations if needed
pub fn get() -> Result<Connection> {
    let db_path = Config::database_path()?;

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;

    // Run migrations
    crate::db::migrations::run_migrations(&conn)?;

    Ok(conn)
}
