use sqlx;
use actix_web::{Result, delete, web, HttpResponse, Error};
use crate::models::pools_models::AccountPools;
use serde::{Serialize, Deserialize};
use crate::helpers::target_pool::target_account_pool;
use uuid::Uuid;
use serde_json;

#[derive(Serialize, Deserialize)]
struct Payload {
  store_id: Uuid,
  store_table: String
}

#[delete("/stores/{account_id}")]
pub async fn delete_store(
  account_pools: web::Data<AccountPools>,
  account_id: web::Path<String>,
  payload: web::Json<Payload>
) -> Result<HttpResponse, Error> {
  let store_id = &payload.store_id;
  let store_table = &payload.store_table;
  let account_id = account_id.into_inner();
  let account_conn = target_account_pool(account_id.clone(), account_pools);

  let store_table = format!("{}_products", store_table);
  let query = format!("DROP TABLE IF EXISTS {}", store_table);

  let delete_products_table = sqlx::query(&query)
    .execute(&account_conn)
    .await;

  match delete_products_table {
    Ok(_) => {
      println!("Products table from {} has been dropped", store_table);
    }

    Err(err) => {
      HttpResponse::InternalServerError().json(serde_json::json!({
        "status": "error",
        "message": format!("Internal Server Error while dropping {} products table: {}", store_table, err),
        "response": []
      }));
    }
  }

  
  let deleted_store = sqlx::query(
    r#"
        DELETE FROM stores
        WHERE id = $1::uuid
        AND account_id = $2::uuid
    "#)
      .bind(store_id)
      .bind(account_id)
      .execute(&account_conn)
      .await;

    match deleted_store {
      Ok(_) => {
        println!("Store {} has been removed from user account", store_table);
      }
      Err(err) => {
        println!("Store could not be removed: {}", err)
        
      }
    }

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": "success",
    "message": "Store has been deleted.",
    "response": []
  })))

}