use sqlx;
use actix_web::{Result, delete, web, HttpResponse, Error, HttpRequest};
use crate::{auth::token_to_user_id::token_to_string_id, models::pools_models::AccountPools};
use serde::{Serialize, Deserialize};
use crate::helpers::target_pool::target_account_pool;
use uuid::Uuid;
use serde_json;

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
) -> Result<HttpResponse, Error> {
  let store_id = &payload.store_id;
  let store_table = &payload.store_table;

  let account_id = token_to_string_id(req);
  
  match account_id {
    Ok(id) => {
        let account_conn = target_account_pool(id.clone(), account_pools).unwrap();
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
            .bind(id)
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
          "status": 200,
          "message": "Store has been deleted.",
          "response": []
        })))
    }

    Err(err) => {
        eprintln!("Error fetching stores: {:?}", err);
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
        "status": 401,
        "message": "Invalid or missing token",
        "response": []
      })))
    }
  }
}