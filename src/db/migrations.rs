use rusqlite::Connection;
use crate::error::Result;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial_schema", include_str!("../../migrations/001_initial_schema.sql")),
    ("002_rename_active_course_to_active", include_str!("../../migrations/002_rename_active_course_to_active.sql")),
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create schema version table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Get current schema version
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Apply pending migrations
    for (idx, (name, sql)) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i64;
        if version > current_version {
            println!("Applying migration {}: {}", version, name);

            // Execute migration SQL
            conn.execute_batch(sql)?;

            // Record migration
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [version],
            )?;

            println!("✓ Migration {} applied successfully", version);
        }
    }

    Ok(())
}
