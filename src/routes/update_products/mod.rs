pub mod build_shopify_query;
use reqwest;

use build_shopify_query::build_shopify_query;
use serde::{Deserialize, Serialize};
use actix_web::{web, put, Result, HttpResponse, Error};
use serde_json;
use crate::{helpers::{modify_types::string_to_uuid, target_pool::target_account_pool}, models::{
  pools_models::AccountPools, products_models::shopify_products::ShopifyProductResponse, store_models::Store
}};
mod parse_shopify_products;
use parse_shopify_products::parse_shopify_products;
mod add_products_to_store;
use add_products_to_store::add_products_to_store;

#[derive(Deserialize, Serialize)]
struct UpdateProductsPayload {
  store_id: String,
  shopify_storefront_store_name: Option<String>,
  shopify_storefront_access_token: Option<String>
}

#[put("/stores/update-products/{account_id}")]
pub async fn update_products(
  account_pools: web::Data<AccountPools>,
  account_id: web::Path<String>,
  payload: web::Json<UpdateProductsPayload>
) -> Result<HttpResponse, Error> {



  let UpdateProductsPayload {
    store_id, 
    shopify_storefront_store_name, 
    shopify_storefront_access_token,
  } = payload.into_inner();

  let account_conn = target_account_pool(account_id.clone(), account_pools);

  let store_uuid = string_to_uuid(store_id);

  let store = sqlx::query_as::<_, Store>(
    "SELECT * FROM stores WHERE id = $1"
  )
    .bind(store_uuid)
    .fetch_one(&account_conn)
    .await        
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Error fetching store")
    })?;

  match store.platform.as_str() {
    "shopify" => {
        let mut limit = 250;
        let storefront_query: String = build_shopify_query(limit);
        let client = reqwest::Client::new();


        let response = client
            .post(&format!("https://{}/api/unstable/graphql.json", shopify_storefront_store_name.unwrap_or("".to_string())))
            .header("Content-Type", "application/json")
            .header("X-Shopify-Storefront-Access-Token", shopify_storefront_access_token.unwrap_or("".to_string()))
            .json(&serde_json::json!({
                "query": storefront_query
            }))
            .send()
            .await
            .map_err(|err| {
              eprintln!("Failed to read response text: {}", err);
              actix_web::error::ErrorInternalServerError("Failed to read Shopify response")
            })?;

        let shopify_products: ShopifyProductResponse = response.json()
            .await
            .map_err(|e| {
                eprintln!("JSON parse error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to convert Shopify response to ShopifyProductResponse type")
            })?;


        let parsed_products = parse_shopify_products(shopify_products)
            .map_err(|e| {
                eprintln!("JSON parse error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to parse Shopify response")
            })?;

        let _response = add_products_to_store(account_conn.clone(), parsed_products.clone(), store.store_table);

        Ok(HttpResponse::Ok().json(serde_json::json!({
          "status": "success",
          "message": "Successfully updated products",
          "response": parsed_products
        })))
    }
    _ => {
        // Handle unknown platforms
        println!("Unknown platform: {}", store.platform);
        Err(
          actix_web::error::ErrorInternalServerError("Error")
        )
    }
  }


  // Ok(HttpResponse::Ok().json(serde_json::json!({
  //   "status": "success",
  //   "message": "Successfully updated products",
  //   "response": []
  // })))
}