use sea_orm::{Database, DatabaseConnection, DbErr};
use crate::paths;

pub async fn get_connection() -> Result<DatabaseConnection, DbErr> {
    let db_path = paths::database_path()
        .map_err(|e| DbErr::Custom(format!("Failed to get database path: {}", e)))?;
    
    // SQLite connection string: sqlite://path/to/db?mode=rwc
    let database_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
    
    Database::connect(database_url).await
}