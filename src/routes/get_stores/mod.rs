use crate::helpers::target_pool::{target_account_pool};
use actix_web::{get, web, HttpResponse, Error};
use crate::models::{pools_models::AccountPools, store_models::Store};
use serde_json;

#[get("/stores/{account_id}")]
pub async fn get_stores(
  account_pools: web::Data<AccountPools>,
  account_id: web::Path<String>,  
) -> Result<HttpResponse, Error>{
  let id_string = account_id.into_inner();

  let account_conn = target_account_pool(id_string, account_pools);

  let stores = sqlx::query_as::<_, Store>("SELECT * from stores")
      .fetch_all(&account_conn)
      .await
      .map_err(|err| {
          HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("Internal server error: {}", err),
            "response": []
          }));
          actix_web::error::ErrorInternalServerError(format!("Failed while fetching stores data: {}", err))
      })?;

  let stores_res = if stores.len() != 0 {stores} else {Vec::new()};

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": "success",
    "message": "Successfully fetched stores",
    "response": stores_res
  })))
}