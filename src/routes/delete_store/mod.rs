use sqlx;
use actix_web::{delete, web, HttpResponse, HttpRequest};
use crate::{ models::pools_models::AccountPools};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use serde_json;
use crate::helpers::init_account_connection::init_account_connection;

#[derive(Serialize, Deserialize)]
struct DeleteStorePayload {
  store_id: Uuid,
  store_table: String
}

#[delete("/stores")]
pub async fn delete_store(
  account_pools: web::Data<AccountPools>,
  payload: web::Json<DeleteStorePayload>,
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

  let DeleteStorePayload {store_id, store_table} = payload.into_inner();

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
  let query = format!("DROP TABLE IF EXISTS {}_products", store_table);

  let delete_products_table = sqlx::query(&query)
    .execute(&mut *transaction)
    .await;

  if let Err(err) = delete_products_table {
      eprintln!("Failed to drop products table ({}): {}", store_table, err);

      return HttpResponse::InternalServerError().json(serde_json::json!({
          "status": 500,
          "message": "Failed to delete store (products table).",
          "response": []
      }));
  }

  // HANDLES DROPPING EMBEDDINGS TABLE
  let embeddings_table_query = format!("DROP TABLE IF EXISTS {}_embeddings", store_table);
  
  let drop_embeddings_table_query = sqlx::query(&embeddings_table_query)
    .execute(&mut *transaction)
    .await;

  if let Err(err) = drop_embeddings_table_query {
      eprintln!("Failed to drop products table ({}): {}", store_table, err);

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
        AND account_id = $2::uuid
    "#)
      .bind(store_id)
      .bind(&account_id)
      .execute(&mut *transaction)
      .await;

  if let Err(err) = deleted_store {
      eprintln!("No store found to delete for store_id: {}: {}", store_id, err);
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
              "message": format!("Store '{}' and its associated data successfully deleted.", store_table),
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