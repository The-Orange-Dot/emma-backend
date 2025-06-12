use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;
use crate::helpers::get_account_psql_link::get_account_psql_link;
use crate::models::pools_models::{PoolWrapper, AccountPools};

impl AccountPools {
    pub fn new() -> Self {
        AccountPools(Arc::new(RwLock::new(HashMap::new())))
    }

    pub async fn get_pool(
        &self,
        account_id: Uuid,
        username: &str,
        db_password: &str,
    ) -> Result<Pool<Postgres>, std::io::Error> {
        {
            let mut pools = self.0.write()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire write lock"))?;
            
            if let Some(wrapper) = pools.get_mut(&account_id) {
                wrapper.last_used = Instant::now();
                return Ok(wrapper.pool.clone());
            }
        }

        let postgres_url = std::env::var("POSTGRES_URL")
            .expect("Postgres URL has not been set for initializing account connections");

        let account_db_url = get_account_psql_link(
            username.to_string(), 
            db_password.to_string(), 
            postgres_url
        );

        println!("DEBUG: {:?}", account_db_url);

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .idle_timeout(Duration::from_secs(300))
            .test_before_acquire(true)
            .connect(&account_db_url)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let mut pools = self.0.write()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire write lock"))?;
        
        pools.insert(account_id, PoolWrapper {
            pool: pool.clone(),
            last_used: Instant::now(),
        });

        Ok(pool)
    }

    // pub fn cleanup_idle_pools(&self, idle_timeout: Duration) {
    //     let mut pools = match self.0.write() {
    //         Ok(pools) => pools,
    //         Err(e) => {
    //             log::error!("Failed to acquire write lock for cleanup: {}", e);
    //             return;
    //         }
    //     };

    //     let now = Instant::now();
    //     let before_count = pools.len();
        
    //     pools.retain(|id, wrapper| {
    //         let idle_for = now.duration_since(wrapper.last_used);
    //         if idle_for >= idle_timeout {
    //             log::info!("Cleaning up idle pool for account {} (idle for {:?})", id, idle_for);
    //             false
    //         } else {
    //             true
    //         }
    //     });

    //     let after_count = pools.len();
    //     if before_count != after_count {
    //         log::info!("Pool cleanup completed: {} -> {} pools", before_count, after_count);
    //     }
    // }
}