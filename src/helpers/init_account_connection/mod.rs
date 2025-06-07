use crate::helpers::target_pool::target_account_pool;
use actix_web::{HttpRequest, HttpResponse,};
use sqlx::{Pool,Postgres};
use crate::auth::token_to_user_id::{token_to_string_id};
use crate::models::pools_models::{AccountPools};
use actix_web::web::Data;

pub async fn init_account_connection(req: HttpRequest, account_pools: Data<AccountPools>) -> Result<(String, Pool<Postgres>), HttpResponse> {
    
    let account_id = match token_to_string_id(req) {
        Ok(id) => id,
        Err(err) => {
            println!("Failed to parse token to string: {:?}", err);
            return Err(HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "error": format!("Invalid token: {:?}", err)
            })))
        }
    };

    let pool = match target_account_pool(account_id.clone(), account_pools).await {
        Ok(pool) => pool,
        Err(err) => {
            println!("Failed to connect to user pool: {:?}", err);
            return Err(HttpResponse::NotFound().json(serde_json::json!({
                "status": 404,
                "error": format!("Database connection failed: {}", err)
            })))
        }
    };   

    Ok((account_id, pool))
}