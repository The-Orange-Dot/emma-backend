use sqlx::{PgPool};
use std::error::Error;

pub async fn create_store_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .execute(pool)
        .await?;

    Ok(())
}