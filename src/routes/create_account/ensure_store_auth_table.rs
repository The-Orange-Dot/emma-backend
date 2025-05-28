use sqlx::postgres::{PgPool};
use std::error::Error;

pub async fn ensure_store_auth_table(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email VARCHAR(255) CHECK (email ~* '^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$'),
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            status VARCHAR(50) DEFAULT 'inactive',
            plan VARCHAR(50),
            credits INT DEFAULT 0,
            password VARCHAR(255) NOT NULL,
            db_password VARCHAR(255) NOT NULL,
            subscription_ends TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_login_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create index for faster lookups
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_accounts_username ON accounts(username)"
    )
    .execute(pool)
    .await?;

    Ok(())
}