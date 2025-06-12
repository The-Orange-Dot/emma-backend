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

    // Get read access to the pools
    let pools = account_pools.0.read()
        .map_err(|e| {
            log::error!("Failed to acquire read lock: {}", e);
            actix_web::error::ErrorInternalServerError("Database busy")
        })?;

    // Find and return the pool if it exists
    let _pool_found = pools.get(&account_uuid)
        .map(|wrapper| wrapper.pool.clone())
        .ok_or_else(|| {
            println!("Account pool not found");
        });

    let admin_conn: Pool<Postgres> = target_admin_pool(admin_pool);

    let _account = match sqlx::query_as::<_, Account>("SELECT * FROM accounts where id = $1")
        .bind(account_id)
        .fetch_one(&admin_conn)
        .await
        {
            Ok(account) => {

                dotenv::dotenv().ok();
                let database_url = std::env::var("POSTGRES_URL").unwrap();                        
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
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

                account_pools.0.write().unwrap().insert(
                        account.id,
                        PoolWrapper {
                            pool: pool.clone(),
                            last_used: std::time::Instant::now(),
                        }
                    );      
               return Ok(pool)

            }
            Err(_err) => {
                return Err(actix_web::error::ErrorNotFound("Account pool not found"))
            }
        };   
}

pub fn target_admin_pool(
    admin_pool: web::Data<AdminPool>
) -> Pool<Postgres> {
    admin_pool.0.clone()
}