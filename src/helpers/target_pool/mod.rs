use actix_web::{web, Error};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::pools_models::{AccountPools, AdminPool};

// First, let's properly define our PoolWrapper and AccountPools

pub async fn target_account_pool(
    account_id: String,
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
    pools.get(&account_uuid)
        .map(|wrapper| wrapper.pool.clone())
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account pool not found"))
}

pub fn target_admin_pool(
    admin_pool: web::Data<AdminPool>
) -> Pool<Postgres> {
    admin_pool.0.clone()
}