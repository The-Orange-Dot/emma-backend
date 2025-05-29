pub mod build_shopify_query;
use reqwest;

use build_shopify_query::build_shopify_query;
use serde::{Deserialize, Serialize};
use actix_web::{web, put, Result, HttpResponse, Error};
use serde_json;
use crate::{
  helpers::{
    modify_types::string_to_uuid, 
    target_pool::target_account_pool
  }, 
  models::{
    pools_models::AccountPools, 
    products_models::shopify_products::ShopifyProductResponse, 
    store_models::Store
  }
};
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
        let _update_response = sqlx::query(
            r#"
                UPDATE stores 
                SET shopify_storefront_store_name = $1,
                shopify_storefront_access_token = $2
                WHERE id = $3;
            "#)
            .bind(&shopify_storefront_store_name)
            .bind(&shopify_storefront_access_token)
            .bind(store_uuid)
            .execute(&account_conn)
            .await
            .map_err(|err| {
                println!("Error updating Shopify access token columns: {}", err)
            });


        let mut limit: usize = 250;
        let shopify_store_name = shopify_storefront_store_name.unwrap_or("".to_string());
        let shopify_access_token = shopify_storefront_access_token.unwrap_or("".to_string());

        loop {
          let storefront_query: String = build_shopify_query(limit);
          let client = reqwest::Client::new();
              
          let response = client
              .post(&format!("https://{}/api/unstable/graphql.json", &shopify_store_name))
              .header("Content-Type", "application/json")
              .header("X-Shopify-Storefront-Access-Token", &shopify_access_token)
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

          println!("NUMBER OF PRODUCTS: {:?}", shopify_products.data.products.nodes.len());

          let number_of_products = shopify_products.data.products.nodes.len();

          let parsed_products = parse_shopify_products(shopify_products, store_uuid)
              .map_err(|e| {
                  eprintln!("JSON parse error: {}", e);
                  actix_web::error::ErrorInternalServerError("Failed to parse Shopify response")
              })?;

          let _response = add_products_to_store(
            account_conn.clone(), 
            parsed_products.clone(), 
            store.clone().store_table,
          )
            .await
            .map_err(|err| {
              println!("Error adding products to '{:?}_products' table: {:?}", store.store_table, err)
            });

            if number_of_products < 250 {
                println!("{:?}", "No more products to fetch. Ending API fetching loop.");
                break;
            } else {
                println!("{:?}", "Over 250 products fetched, continuing to fetch products.");
                limit += 250;  
            }
        }

        Ok(HttpResponse::Ok().json(serde_json::json!({
          "status": "success",
          "message": "Successfully updated products",
          "response": []
        })))
    }
    _ => {
        // Handle unknown platforms
        println!("Unknown platform: {:?}", store.platform);
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