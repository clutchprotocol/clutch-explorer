use crate::explorer::error::ExplorerError;
use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> Result<(), ExplorerError> {
    let migration_sql = include_str!("../../migrations/0001_init_explorer_schema.sql");
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }

        sqlx::query(statement)
            .execute(&mut *tx)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

    Ok(())
}

pub async fn cleanup_database(pool: &PgPool) -> Result<(), ExplorerError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

    // Truncate all tables in reverse order of dependencies if any
    sqlx::query("TRUNCATE TABLE transactions, blocks, accounts, validators, indexer_cursor CASCADE")
        .execute(&mut *tx)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

    Ok(())
}
