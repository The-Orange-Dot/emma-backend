use actix_web::{post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use upload_csv_to_database::upload_csv_to_database;
use crate::helpers::to_snake_case::to_snake_case;
use crate::models::pools_models::{ AccountPools};
use crate::helpers::modify_types::string_to_uuid;
use uuid::Uuid;
mod upload_csv_to_database;
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

  let Payload {store_name, 
    domain, 
    platform, 
    shopify_storefront_store_name,
    shopify_storefront_access_token,
    csv
  } = payload.into_inner();

  let store_table_name = to_snake_case(&store_name);
  let account_uuid = string_to_uuid(account_id);
  let store_uuid = Uuid::new_v4();
  let insert_new_store = sqlx::query(
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
    .bind(&account_uuid)
    .bind(&store_name)
    .bind(&store_table_name)
    .bind(domain)
    .bind(&platform)
    .bind("")
    .bind(shopify_storefront_store_name)
    .bind(shopify_storefront_access_token)
    .execute(&mut *transaction)
    .await;

    if let Err(err) = insert_new_store {
      eprint!("Error inserting new store into stores table: {}", err);
      return HttpResponse::InternalServerError().json(serde_json::json!({
          "status": 500,
          "message": format!("Failed to create store: {}", err),
          "response": []
      }));   
    }


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
        let _ = upload_csv_to_database(pool.clone(), csv.clone(), store_table_name.clone(), store_uuid)
            .await;
        }

        // ADD SHOPIFY HERE

        println!("Product table for '{}' has been created", &store_table_name);
    },
    Err(err) => {
        eprintln!("Failed to create products table '{}': {}", store_name, err);
    }
  }

  let add_table_to_product_store = sqlx::query(
    "
      INSERT INTO store_products (store_id, products_table_name)
      VALUES ($1, $2)
      ON CONFLICT (store_id, products_table_name) DO NOTHING;
    "
  )
    .bind(&store_uuid)
    .bind(&table_name)
    .execute(&mut *transaction)
    .await;

  if let Err(err) = add_table_to_product_store {
    eprintln!("Failed to add table to product_store: {}", err);
    return HttpResponse::InternalServerError().json(serde_json::json!({
        "status": 500,
        "message": "Failed to initiate database operation.",
        "response": []
    }));    
  }

  let snake_case_store_name = to_snake_case(&store_name);

  match transaction.commit().await {
      Ok(_) => {
          HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "message": "Store created",
            "response": {
              "store_name": snake_case_store_name, 
              "store_id": store_uuid.to_string()
            }
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