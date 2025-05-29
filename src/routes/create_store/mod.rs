use actix_web::{post, HttpResponse, web, Result, Error};
use serde::{Deserialize, Serialize};
use crate::helpers::to_snake_case::to_snake_case;
use crate::init::update_embed_data::{update_embed_data};
use crate::models::pools_models::{ AccountPools};
use crate::helpers::target_pool::{ target_account_pool};
use crate::helpers::modify_types::string_to_uuid;

#[derive(Serialize, Deserialize)]
struct Params {
  account_id: String
}

#[derive(Serialize, Deserialize)]
struct Payload {
store_name: String,
domain: String,
platform: String,
}

#[post("/stores/{account_id}")]
pub async fn create_store(
    account_pools: web::Data<AccountPools>,
    account_id: web::Path<String>,  
    payload: web::Json<Payload> 
  ) -> Result<HttpResponse, Error> {
    let account_conn = target_account_pool(account_id.to_string(), account_pools);
    let Payload {store_name, domain, platform} = payload.into_inner();
    let id: String = account_id.into_inner();
    let store_table_name = to_snake_case(&store_name);

    let account_uuid = string_to_uuid(id);



    let new_store = sqlx::query(
      "INSERT INTO stores (account_id, store_name, store_table, domain, platform, sys_prompt) VALUES ($1, $2, $3, $4, $5, $6)"
    )
      .bind(account_uuid)
      .bind(&store_name)
      .bind(&store_table_name)
      .bind(domain)
      .bind(platform)
      .bind("")
      .execute(&account_conn)
      .await
      .map_err(|err| {
        HttpResponse::InternalServerError().json(serde_json::json!({
          "status": "error",
          "message": format!("Internal server error: {}", err),
          "response": []
        }));
        actix_web::error::ErrorInternalServerError(format!("Error creating new store: {}.", err))
      })?;

  if new_store.rows_affected() == 0 {
      return Ok(HttpResponse::NotFound().json(serde_json::json!({
          "status": "error",
          "message": format!("Error creating store: {}", &store_name),
          "response": []
      })));
  }

  let table_name = format!("{}_products", store_table_name);

  let create_new_product_table_query = format!("
  CREATE TABLE IF NOT EXISTS {} (
          id BIGSERIAL PRIMARY KEY,
          store_id UUID NOT NULL,
          created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
          updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
          name VARCHAR(255) NOT NULL,
          price NUMERIC(10,2) CHECK (price >= 0),
          vendor VARCHAR(255),
          image VARCHAR(255),
          handle VARCHAR(255) NOT NULL,
          description TEXT,
          seo_title VARCHAR(255),
          seo_description VARCHAR(255),
          status VARCHAR(50) NOT NULL,
          published VARCHAR(50),
          category VARCHAR(50),
          tags VARCHAR(255),
          type VARCHAR(50),
          embedding vector(768),
          UNIQUE(handle)
      )
      ", &table_name);

  let new_products_table = sqlx::query(&create_new_product_table_query)
    .execute(&account_conn)
    .await;

  match new_products_table {
    Ok(_) => {
        update_embed_data(account_conn.clone(), 60 * 60 * 12, store_table_name.clone())
          .await;
        println!("Product table for '{}' has been created", &store_table_name);
    },
    Err(err) => {
        eprintln!("Failed to create products table '{}': {}", store_name, err);
    }
  }

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": "success",
    "message": "Store created",
    "response": []
  })))
}