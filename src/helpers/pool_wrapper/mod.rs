use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;
use crate::helpers::get_account_psql_link::get_account_psql_link;

struct PoolWrapper {
    pool: Pool<Postgres>,
    last_used: Instant,
}

pub struct AccountPools(Arc<RwLock<HashMap<Uuid, PoolWrapper>>>);

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
        // First try to get and update an existing pool
        {
            let mut pools = self.0.write()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire write lock"))?;
            
            if let Some(wrapper) = pools.get_mut(&account_id) {
                wrapper.last_used = Instant::now();
                return Ok(wrapper.pool.clone());
            }
        }

        // If no pool exists, create a new one
        let postgres_url = std::env::var("POSTGRES_URL")
            .expect("Postgres URL has not been set for initializing account connections");

        let account_db_url = get_account_psql_link(
            username.to_string(), 
            db_password.to_string(), 
            postgres_url
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .idle_timeout(Duration::from_secs(300))
            .test_before_acquire(true)
            .connect(&account_db_url)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // Insert the new pool
        let mut pools = self.0.write()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire write lock"))?;
        
        pools.insert(account_id, PoolWrapper {
            pool: pool.clone(),
            last_used: Instant::now(),
        });

        Ok(pool)
    }

    pub fn cleanup_idle_pools(&self, idle_timeout: Duration) {
        let mut pools = match self.0.write() {
            Ok(pools) => pools,
            Err(_) => return, // Log error in production
        };

        let now = Instant::now();
        pools.retain(|_, wrapper| {
            // Keep pools that have been used recently
            now.duration_since(wrapper.last_used) < idle_timeout
        });
    }
}