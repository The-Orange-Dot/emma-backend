use sqlx::{Pool, Postgres, FromRow, postgres::PgPoolOptions};
use uuid::Uuid;
use super::get_account_psql_link::get_account_psql_link;
use crate::models::pools_models::AccountPools;
use std::collections::HashMap;
use crate::helpers::install_extensions::install_extensions;

#[derive(FromRow)]
struct AccountRes {
  username: String,
  db_password: String,
  id: Uuid
}

pub async fn init_all_account_connections(
    pool: Pool<Postgres>,
    admin_url: String
) -> Result<AccountPools, std::io::Error> {
    dotenv::dotenv().ok();

    let accounts = sqlx::query_as::<_, AccountRes>(
        "SELECT id, username, db_password FROM accounts"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let mut account_pools = HashMap::new();
    
    for account in accounts {
        let database_url = std::env::var("POSTGRES_URL")
            .expect("Postgres URL has not been set for initializing account connections");
    
    let _ = install_extensions(&admin_url, &account.username)
            .await;

        let store_db_url = get_account_psql_link(
            account.username.clone(), 
            account.db_password.clone(), 
            database_url
        );

        let store_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&store_db_url)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            println!("Connected to {} account pool", account.username);

        account_pools.insert(account.id, store_pool);
    }
    
    Ok(AccountPools(account_pools))
}