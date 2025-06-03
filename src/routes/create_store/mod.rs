use actix_web::{post, web, Error, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use upload_csv_to_database::upload_csv_to_database;
use crate::auth::token_to_user_id::{token_to_string_id};
use crate::helpers::to_snake_case::to_snake_case;
use crate::models::pools_models::{ AccountPools};
use crate::helpers::target_pool::{ target_account_pool};
use crate::helpers::modify_types::string_to_uuid;
use uuid::Uuid;
mod upload_csv_to_database;
use crate::routes::create_store::upload_csv_to_database::{CSV};

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
  ) -> Result<HttpResponse, Error> {
    let account_uuid = token_to_string_id(req);

    match account_uuid {
      Ok(id) => {
          let Payload {store_name, 
            domain, 
            platform, 
            shopify_storefront_store_name,
            shopify_storefront_access_token,
            csv
          } = payload.into_inner();
          let store_table_name = to_snake_case(&store_name);
          let account_uuid = string_to_uuid(id.clone());
          let account_conn = target_account_pool(id, account_pools)?;
          let store_uuid = Uuid::new_v4();

          let _new_store = sqlx::query(
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
          .execute(&account_conn)
          .await
          .map_err(|err| {
              println!("{:?}", err);
              HttpResponse::InternalServerError().json(serde_json::json!({
                  "status": 500,
                  "message": format!("Internal server error: {}", err),
                  "response": []
              }));
              actix_web::error::ErrorInternalServerError(format!("Error creating new store: {}.", err))
          }).unwrap();



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
            .execute(&account_conn)
            .await;

          let _add_table_to_product_store = sqlx::query(
            "
              INSERT INTO store_products (store_id, products_table_name)
              VALUES ($1, $2)
              ON CONFLICT (store_id, products_table_name) DO NOTHING;
            "
          )
            .bind(&store_uuid)
            .bind(&table_name)
            .execute(&account_conn)
            .await;

          match new_products_table {
            Ok(_) => {

                if platform == "csv" {
                let _ = upload_csv_to_database(account_conn.clone(), csv.clone(), store_table_name.clone(), store_uuid)
                    .await;
                }

                println!("Product table for '{}' has been created", &store_table_name);
            },
            Err(err) => {
                eprintln!("Failed to create products table '{}': {}", store_name, err);
            }
          }

          let snake_case_store_name = to_snake_case(&store_name);


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
        eprintln!("Error fetching stores: {:?}", err);
          Ok(HttpResponse::Unauthorized().json(serde_json::json!({
          "status": 401,
          "message": "Invalid or missing token",
          "response": []
        })))
      }
    }
    
}