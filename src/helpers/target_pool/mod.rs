use sqlx::{Pool, Postgres};
use actix_web::{web, Error};
use crate::models::pools_models::{AccountPools, AdminPool};
use crate::helpers::modify_types::string_to_uuid;

pub fn target_account_pool(
    account_id: String,
    account_pools: web::Data<AccountPools>,
) -> Result<Pool<Postgres>, Error> {
    let account_uuid = string_to_uuid(account_id);
    
    // Acquire read lock first
    let pools = account_pools.0.read()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire read lock"))?;
    
    pools.get(&account_uuid)
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))
}

pub fn target_admin_pool(
    admin_pool: web::Data<AdminPool>
) -> Pool<Postgres> {
    admin_pool.0.clone()
}