use sqlx;
use actix_web::{delete, web, HttpResponse, HttpRequest};
use crate::models::{pools_models::AccountPools, store_models::Store};
use serde::{Serialize, Deserialize};
use serde_json;
use crate::helpers::init_account_connection::init_account_connection;
use crate::helpers::modify_types::string_to_uuid;

#[derive(Serialize, Deserialize)]
struct DeleteStorePayload {
  store_table: String
}

#[delete("/store/{store_id}")]
pub async fn delete_store(
  account_pools: web::Data<AccountPools>,
  req: HttpRequest,
  path: web::Path<String>
) -> HttpResponse {
  let (_account_id, pool) = match init_account_connection(req, account_pools).await {
      Ok(res) => res,
      Err(err) => {
          return HttpResponse::BadRequest().json(serde_json::json!({
              "status": 400,
              "error": format!("Invalid token: {:?}", err)
          }));
      }
  };

  let store_uuid = string_to_uuid(path.into_inner());

  // Fetches the account first
  let found_store = match sqlx::query_as::<_, Store>("SELECT * FROM stores WHERE id = $1")
    .bind(store_uuid)
    .fetch_one(&pool)
    .await
    {
      Ok(res) => res,
      Err(err) => {
        return HttpResponse::NotFound().json(serde_json::json!({
          "status": 404,
          "message": format!("No account found to delete: {}", err),
          "response": []
        }))
      }
    };

  let mut transaction = match pool.begin().await {
      Ok(t) => t,
      Err(e) => {
          eprintln!("Failed to begin transaction: {}", e);
          return HttpResponse::InternalServerError().json(serde_json::json!({
              "status": 500,
              "message": "Failed to initiate database operation.",
              "response": []
          }));
      }
  };
  
  // HANDLES DROPPING PRODUCTS TABLE
  let query = format!("DROP TABLE IF EXISTS {}_products", found_store.store_table);

  let delete_products_table = sqlx::query(&query)
    .execute(&mut *transaction)
    .await;

  if let Err(err) = delete_products_table {
      eprintln!("Failed to drop products table ({}): {}", found_store.store_table, err);

      return HttpResponse::InternalServerError().json(serde_json::json!({
          "status": 500,
          "message": "Failed to delete store (products table).",
          "response": []
      }));
  }

  // HANDLES DROPPING EMBEDDINGS TABLE
  let embeddings_table_query = format!("DROP TABLE IF EXISTS {}_embeddings", found_store.store_table);
  
  let drop_embeddings_table_query = sqlx::query(&embeddings_table_query)
    .execute(&mut *transaction)
    .await;

  if let Err(err) = drop_embeddings_table_query {
      eprintln!("Failed to drop products table ({}): {}", found_store.store_table, err);

      return HttpResponse::InternalServerError().json(serde_json::json!({
          "status": 500,
          "message": "Failed to delete store (embeddings table).",
          "response": []
      }));
  }  

  // HANDLES REMOVING STORE FROM ACCOUNT STORES TABLE
  let deleted_store = sqlx::query(
    r#"
        DELETE FROM stores
        WHERE id = $1::uuid
    "#)
      .bind(store_uuid)
      .execute(&mut *transaction)
      .await;

  if let Err(err) = deleted_store {
      eprintln!("No store found to delete for store_id: {}: {}", store_uuid, err);
      return HttpResponse::NotFound().json(serde_json::json!({
          "status": 404,
          "message": "Store not found or unauthorized.",
          "response": []
      }));
  }  

  match transaction.commit().await {
      Ok(_) => {
          HttpResponse::Ok().json(serde_json::json!({
              "status": 200,
              "message": format!("Store '{}' and its associated data successfully deleted.", found_store.store_table),
              "response": []
          }))
      }
      Err(e) => {
          eprintln!("Failed to commit transaction: {}", e);
          HttpResponse::InternalServerError().json(serde_json::json!({
              "status": 500,
              "message": "Failed to finalize store deletion.",
              "response": []
          }))
      }
  }

}