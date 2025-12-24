use sea_orm::{DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

/// Run all pending migrations using SeaORM's migration system
/// This automatically tracks which migrations have been applied
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    migration::Migrator::up(db, None).await
}
