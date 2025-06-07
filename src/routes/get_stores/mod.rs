
use actix_web::{get, web, HttpRequest, HttpResponse};
use crate::models::{pools_models::AccountPools, store_models::Store};
use serde_json;
use crate::helpers::init_account_connection::init_account_connection;

#[get("/stores")]
pub async fn get_stores(
    req: HttpRequest,
    account_pools: web::Data<AccountPools>,
) -> HttpResponse {
    let (_account_id, pool) = match init_account_connection(req, account_pools).await {
        Ok(res) => res,
        Err(err) => {
            println!("Failed to initialize account connection: {:?}", err);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "error": format!("Invalid token: {:?}", err)
            }));
        }
    };

    match sqlx::query_as::<_, Store>("SELECT * FROM stores")
        .fetch_all(&pool)
        .await
    {
        Ok(stores) => HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "message": "Successfully fetched stores",
            "response": stores
        })),
        Err(err) => {
            log::error!("Error fetching stores: {}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "error": "Failed to fetch stores"
            }))
        }
    }
  
}