use actix_web::{post, web, HttpRequest, HttpResponse,    
  error::{ErrorInternalServerError, ErrorBadRequest}
};
use serde::{Deserialize, Serialize};
use upload_csv_to_database::upload_csv_to_database;
use upload_shopify_to_database::upload_shopify_to_database;
use crate::helpers::to_snake_case::to_snake_case;
use crate::models::pools_models::{AccountPools, AdminPool};
use crate::helpers::modify_types::string_to_uuid;
use uuid::Uuid;
mod upload_csv_to_database;
mod upload_shopify_to_database;
use crate::routes::create_store::upload_csv_to_database::{CSV};
use crate::helpers::init_account_connection::init_account_connection;

#[derive(Serialize, Deserialize)]
struct Payload {
store_name: String,
domain: String,
platform: String,
shopify_storefront_store_name: String,
shopify_storefront_access_token: String,
csv: Vec<CSV>
}

#[post("/stores")]
pub async fn create_store(
    account_pools: web::Data<AccountPools>,
    payload: web::Json<Payload> ,
    admin_pool: web::Data<AdminPool>,
    req: HttpRequest
  ) -> Result<HttpResponse, actix_web::Error> {
    let (account_id, pool) = init_account_connection(req, admin_pool, account_pools)
    .await
    .map_err(|err| {
        ErrorBadRequest(format!("Failed to init account connection: {:?}", err))
    })?;


  let mut transaction = pool.begin()
    .await
    .map_err(|err| {
        ErrorInternalServerError(format!("Failed to initiate database operation: {:?}", err))
    })?; 

  let Payload {store_name, 
    domain, 
    platform, 
    shopify_storefront_store_name,
    shopify_storefront_access_token,
    csv
  } = payload.into_inner();

  let account_uuid = string_to_uuid(account_id)
    .map_err(|_err| {
      ErrorBadRequest("Invalid account id")
    })?;

  let store_table_name = to_snake_case(&store_name);
  let store_uuid = Uuid::new_v4();
  let _insert_new_store = sqlx::query(
      "
        INSERT INTO stores (
        id, account_id, store_name, 
        store_table, domain, 
        platform, sys_prompt, 
        shopify_storefront_access_token, 
        shopify_storefront_store_name) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (store_name, store_table, domain) DO NOTHING;
      "
  )
    .bind(&store_uuid)
    .bind(account_uuid)
    .bind(&store_name)
    .bind(&store_table_name)
    .bind(domain)
    .bind(&platform)
    .bind("")
    .bind(&shopify_storefront_store_name)
    .bind(&shopify_storefront_access_token)
    .execute(&mut *transaction)
    .await
    .map_err(|err| {
      ErrorInternalServerError(format!("Failed to create store: {}", err))
    })?;

  // CREATES PRODUCTS TABLE
  let table_name = format!("{}_products", store_table_name);
  let create_new_product_table_query = format!("
  CREATE TABLE IF NOT EXISTS {} (
          id BIGSERIAL PRIMARY KEY,
          store_id UUID NOT NULL,
          created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
          updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
          name VARCHAR(256) NOT NULL,
          price NUMERIC(10,2) CHECK (price >= 0),
          vendor VARCHAR(257),
          image VARCHAR(500),
          handle VARCHAR(259) NOT NULL,
          description VARCHAR(10000),
          seo_title VARCHAR(260),
          seo_description VARCHAR(270),
          status VARCHAR(100) NOT NULL,
          published VARCHAR(50),
          category VARCHAR(255),
          tags VARCHAR(1000),
          type VARCHAR(100),
          product_url VARCHAR(300)
      )
      ", &table_name);

  let new_products_table = sqlx::query(&create_new_product_table_query)
    .execute(&mut *transaction)
    .await;

  match new_products_table {
    Ok(_) => {

        if platform == "csv" {
          let _upload_from_csv = upload_csv_to_database(&mut *transaction, csv.clone(), store_table_name.clone(), store_uuid)
              .await;
        } else if platform == "shopify" {
          let _upload_from_shopify = upload_shopify_to_database(
            &mut *transaction, 
            shopify_storefront_access_token, 
            shopify_storefront_store_name, 
            store_uuid,
            store_table_name.clone()
          )
              .await;
        }

    },
    Err(err) => {
        eprintln!("Failed to create products table '{}': {}", store_name, err);
    }
  }

  let _add_table_to_product_store = sqlx::query(
    "
      INSERT INTO store_products (store_id, products_table_name)
      VALUES ($1, $2)
      ON CONFLICT (store_id, products_table_name) DO NOTHING;
    "
  )
    .bind(&store_uuid)
    .bind(&table_name)
    .execute(&mut *transaction)
    .await
    .map_err(|err| {
      ErrorInternalServerError(format!("Failed to initiate database operation: {}", err))
    })?;

    


  let snake_case_store_name = to_snake_case(&store_name);

  match transaction.commit().await {
      Ok(_) => {
          Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "message": "Store created",
            "response": {
              "store_name": snake_case_store_name, 
              "store_id": store_uuid.to_string()
            }
          })))
      }
      Err(err) => {
          eprintln!("Failed to commit transaction: {}", err);
          Err(ErrorInternalServerError(format!("Failed to finalize store deletion: {}", err)))
      }
  }  
}