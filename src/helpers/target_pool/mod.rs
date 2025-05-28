use sqlx::{Pool, Postgres};
use uuid::Uuid;
use actix_web::{web};
use crate::models::pools_models::{AccountPools, AdminPool};

pub fn target_account_pool(
    account_id: String,
    account_pools: web::Data<AccountPools>,
) -> Pool<Postgres> {
    let account_uuid = Uuid::parse_str(&account_id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))
        .expect("Failed to parse UUID");
    
    account_pools.0.get(&account_uuid)
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))
        .expect("Account pool not found")
        .clone()
}

pub fn target_admin_pool(
  admin_pool: web::Data<AdminPool>
) -> Pool<Postgres> {
    let admin_conn = &admin_pool.0; 

    admin_conn.clone()
  }

