use sea_orm::{Database, DatabaseConnection, DbErr};
use crate::paths;
use crate::db::migrations;

pub async fn get_connection() -> Result<DatabaseConnection, DbErr> {
    let db_path = paths::database_path()
        .map_err(|e| DbErr::Custom(format!("Failed to get database path: {}", e)))?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DbErr::Custom(format!("Failed to create database directory: {}", e)))?;
    }

    // SQLite connection string: sqlite://path/to/db?mode=rwc
    let database_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let db = Database::connect(database_url).await?;

    // Run pending migrations
    migrations::run_migrations(&db).await?;

    Ok(db)
}