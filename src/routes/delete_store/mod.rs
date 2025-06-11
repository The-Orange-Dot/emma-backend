use sqlx;
use actix_web::{delete, error::{ErrorInternalServerError, ErrorNotFound}, web, HttpRequest, HttpResponse};
use crate::models::{pools_models::AccountPools, store_models::Store};
use serde::{Serialize, Deserialize};
use serde_json;
use crate::helpers::init_account_connection::init_account_connection;
use crate::helpers::modify_types::string_to_uuid;
use actix_web::error::{ErrorBadRequest}; 

#[derive(Serialize, Deserialize)]
struct DeleteStorePayload {
  store_table: String
}

#[delete("/store/{store_id}")]
pub async fn delete_store(
  account_pools: web::Data<AccountPools>,
  req: HttpRequest,
  path: web::Path<String>
) -> Result<HttpResponse, actix_web::Error> {
  let (_account_id, pool) = init_account_connection(req, account_pools).await 
    .map_err(|err| {
      ErrorBadRequest(format!("Invalid user ID format in token claims: {:?}", err))
    })?;


  let store_uuid = string_to_uuid(path.into_inner())
    .map_err(|err| { 
      eprintln!("Failed to parse UUID from token claims: {:?}", err);
      ErrorBadRequest("Invalid user ID format in token claims")
    })?;

  // Fetches the account first
  let found_store = sqlx::query_as::<_, Store>("SELECT * FROM stores WHERE id = $1")
    .bind(store_uuid)
    .fetch_one(&pool)
    .await
    .map_err(|err| {
      ErrorNotFound(format!("No account found to delete: {}", err))
    })?;


  let mut transaction = pool.begin()
    .await 
    .map_err(|err| {
      ErrorInternalServerError(format!("Failed to initiate database operation: {}", err))
    })?;

  // HANDLES DROPPING EMBEDDINGS TABLE
  let embeddings_table_query = format!("DROP TABLE IF EXISTS {}_embeddings", found_store.store_table);
  
  let _drop_embeddings_table_query = sqlx::query(&embeddings_table_query)
    .execute(&mut *transaction)
    .await
    .map_err(|err| {
      eprintln!("Failed to delete store (embeddings table): {}", err);
      ErrorInternalServerError(format!("Failed to delete store (embeddings table): {}", err))
    })?;    

  // HANDLES REMOVING STORE FROM ACCOUNT STORES TABLE
  let _deleted_store = sqlx::query(
    r#"
        DELETE FROM stores
        WHERE id = $1::uuid
    "#)
      .bind(store_uuid)
      .execute(&mut *transaction)
      .await
      .map_err(|err| {
        eprintln!("Store not found or unauthorized: {}", err);
        ErrorNotFound(format!("Store not found or unauthorized: {}", err))
      })?;    

  // HANDLES DROPPING PRODUCTS TABLE
  let query = format!("DROP TABLE IF EXISTS {}_products", found_store.store_table);

  let _delete_products_table = sqlx::query(&query)
    .execute(&mut *transaction)
    .await
    .map_err(|err| {
      eprintln!("Failed to delete store (products table): {}", err);
      ErrorInternalServerError(format!("Failed to delete store (products table): {}", err))
    })?;              

  match transaction.commit().await {
      Ok(_) => {
          Ok(HttpResponse::Ok().json(serde_json::json!({
              "status": 200,
              "message": format!("Store '{}' and its associated data successfully deleted.", found_store.store_table),
              "response": []
          })))
      }
      Err(err) => {
          eprintln!("Failed to commit transaction: {}", err);
          Err(ErrorInternalServerError(format!("Failed to delete store (embeddings table): {}", err)))
      }
  }

}