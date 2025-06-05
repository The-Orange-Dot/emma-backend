use sqlx::{Pool, Postgres, FromRow, postgres::PgPoolOptions };
use uuid::Uuid;
use super::get_account_psql_link::get_account_psql_link;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Instant, Duration};
use crate::models::pools_models::{AccountPools, PoolWrapper};

#[derive(FromRow, Clone)]
pub struct AccountInfo {
    pub id: Uuid,
    pub username: String,
    pub db_password: String,
}

pub async fn init_all_account_connections(
    admin_pool: Pool<Postgres>,
) -> Result<AccountPools, sqlx::Error> {
    // First fetch all account information
    let accounts = sqlx::query_as::<_, AccountInfo>(
        "SELECT id, username, db_password FROM accounts"
    )
    .fetch_all(&admin_pool)
    .await?;

    // Create all connection pools in advance
    let mut pools = HashMap::new();
    
    for account in accounts {
        let pool = create_connection_pool(&account.username, &account.db_password).await?;
        pools.insert(account.id, PoolWrapper {
            pool,
            last_used: Instant::now(),
        });
    }

    // Now wrap in Arc<RwLock> after all pools are created
    Ok(AccountPools(Arc::new(RwLock::new(pools))))
}

async fn create_connection_pool(
    username: &str,
    password: &str,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let postgres_url = std::env::var("POSTGRES_URL")
        .expect("POSTGRES_URL must be set");
    
    let db_url = get_account_psql_link(
        username.to_string(),
        password.to_string(),
        postgres_url
    );

    PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(300))
        .connect(&db_url)
        .await
}