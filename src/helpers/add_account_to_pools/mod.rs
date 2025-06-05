use crate::helpers::{
    install_extensions::install_extensions, 
    get_account_psql_link::get_account_psql_link
};
use crate::models::pools_models::AccountPools;
use uuid::Uuid;
use sqlx::{postgres::PgPoolOptions};
use crate::models::pools_models::PoolWrapper;

pub async fn add_account_to_pools(
    account_pools: &AccountPools,
    admin_url: &str,
    account_id: Uuid,
    username: &str,
    db_password: &str,
    database_url: String
) -> Result<String, std::io::Error> {
    dotenv::dotenv().ok();
    let postgres_url = std::env::var("POSTGRES_URL")
        .expect("Postgres URL has not been set for initializing account connections");
    
    let _ = install_extensions(admin_url, username).await;

    let account_db_url = get_account_psql_link(
        username.to_string(), 
        db_password.to_string(), 
        postgres_url
    );

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&account_db_url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    println!("Connected to {} account pool", username);

    // Create PoolWrapper and insert it
    account_pools.0.write().unwrap().insert(
        account_id,
        PoolWrapper {
            pool,
            last_used: std::time::Instant::now(),
        }
    );
    
    let account_db_url = get_account_psql_link(
        username.to_string(), 
        db_password.to_string(), 
        database_url
    );

    Ok(account_db_url)
}