use actix_web::{web, Error};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use uuid::Uuid;
use crate::models::{account_models::Account, pools_models::{AccountPools, AdminPool}};
use crate::helpers::get_account_psql_link::get_account_psql_link;
use std::time::Duration;
use crate::models::pools_models::PoolWrapper;

pub async fn target_account_pool(
    account_id: String,
    admin_pool: web::Data<AdminPool>,
    account_pools: web::Data<AccountPools>,
) -> Result<Pool<Postgres>, Error> {
    let account_uuid = Uuid::parse_str(&account_id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid account ID format"))?;

    let pools_guard = account_pools.0.read() // Renamed to avoid shadowing 'pools'
        .map_err(|e| {
            log::error!("Failed to acquire read lock: {}", e);
            actix_web::error::ErrorInternalServerError("Database busy")
        })?;

    if let Some(wrapper) = pools_guard.get(&account_uuid) {
        println!("DEBUG: Found existing account pool in cache.");
        return Ok(wrapper.pool.clone());
    }

    println!("DEBUG: Account pool not found in cache. Attempting to create new one.");

    let admin_conn: Pool<Postgres> = target_admin_pool(admin_pool);

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts where id = $1")
        .bind(&account_id)
        .fetch_one(&admin_conn)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch account {}: {}", account_id, err);
            actix_web::error::ErrorNotFound("Account not found or database error")
        })?;

    dotenv::dotenv().ok();
    let database_url = std::env::var("POSTGRES_URL")
        .map_err(|_| actix_web::error::ErrorInternalServerError("POSTGRES_URL not set"))?;
                           
    let account_db_url = get_account_psql_link(
        account.username.to_string(), 
        account.db_password.to_string(), 
        database_url
    );                       
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(300))        
        .connect(&account_db_url)
        .await
        .map_err(|err| {
            log::error!("Failed to connect to account DB for {}: {}", account_id, err);
            actix_web::error::ErrorInternalServerError("Failed to connect to account database")
        })?;

    account_pools.0.write()
        .map_err(|e| {
            log::error!("Failed to acquire write lock for pool insertion: {}", e);
            actix_web::error::ErrorInternalServerError("Database busy for pool caching")
        })?
        .insert(
            account.id,
            PoolWrapper {
                pool: pool.clone(),
                last_used: std::time::Instant::now(),
            }
        );      
    
    Ok(pool)
}

pub fn target_admin_pool(
    admin_pool: web::Data<AdminPool>
) -> Pool<Postgres> {
    admin_pool.0.clone()
}