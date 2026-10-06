use crate::explorer::error::ExplorerError;
use sqlx::PgPool;

/// Run on every start of either binary, each file in its own transaction. Not sqlx-migrate: the
/// files are split on `;`, so every statement must be idempotent and none may contain a `;`.
const MIGRATIONS: [&str; 4] = [
    include_str!("../../migrations/0001_init_explorer_schema.sql"),
    include_str!("../../migrations/0002_referrer_fees.sql"),
    include_str!("../../migrations/0003_account_activity.sql"),
    include_str!("../../migrations/0004_activity_count.sql"),
];

pub async fn run_migrations(pool: &PgPool) -> Result<(), ExplorerError> {
    for migration in MIGRATIONS {
        let mut tx = pool.begin().await?;
        for statement in migration
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            sqlx::query(statement).execute(&mut *tx).await?;
        }
        tx.commit().await?;
    }
    Ok(())
}

pub async fn cleanup_database(pool: &PgPool) -> Result<(), ExplorerError> {
    sqlx::query(
        "TRUNCATE TABLE account_activity, transactions, blocks, accounts, validators, indexer_cursor CASCADE",
    )
    .execute(pool)
    .await?;
    Ok(())
}
