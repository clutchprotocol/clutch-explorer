use crate::explorer::error::ExplorerError;
use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> Result<(), ExplorerError> {
    let migration_sql = include_str!("../../migrations/0001_init_explorer_schema.sql");
    sqlx::query(migration_sql)
        .execute(pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;
    Ok(())
}
