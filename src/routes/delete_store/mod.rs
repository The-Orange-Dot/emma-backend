use sqlx;
use actix_web::{delete, web, HttpResponse, HttpRequest};
use crate::{ models::pools_models::AccountPools};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use serde_json;
use crate::helpers::init_account_connection::init_account_connection;

#[derive(Serialize, Deserialize)]
struct Payload {
  store_id: Uuid,
  store_table: String
}

#[delete("/stores")]
pub async fn delete_store(
  account_pools: web::Data<AccountPools>,
  payload: web::Json<Payload>,
  req: HttpRequest 
) -> HttpResponse {
  let (account_id, pool) = match init_account_connection(req, account_pools).await {
      Ok(res) => res,
      Err(err) => {
          return HttpResponse::BadRequest().json(serde_json::json!({
              "status": 400,
              "error": format!("Invalid token: {:?}", err)
          }));
      }
  };

  let Payload {store_id, store_table} = payload.into_inner();
  
  let store_table = format!("{}_products", store_table);
  let query = format!("DROP TABLE IF EXISTS {}", store_table);

  let delete_products_table = sqlx::query(&query)
    .execute(&pool)
    .await;

  match delete_products_table {
    Ok(_) => {
      println!("Products table from {} has been dropped", store_table);
    }

    Err(err) => {
      HttpResponse::InternalServerError().json(serde_json::json!({
        "status": 500,
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
      .execute(&pool)
      .await;

    match deleted_store {
      Ok(_) => {
        println!("Store {} has been removed from user account", store_table);
      }
      Err(err) => {
        println!("Store could not be removed: {}", err)
        
    }
  }

  HttpResponse::Ok().json(serde_json::json!({
    "status": 200,
    "message": "Store has been deleted.",
    "response": []
  }))

}