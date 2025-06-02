pub mod build_shopify_query;
use reqwest;

use build_shopify_query::build_shopify_query;
use serde::{Deserialize, Serialize};
use actix_web::{put, web, HttpRequest, HttpResponse};
use serde_json;
use crate::{
  auth::token_to_user_id::token_to_string_id, helpers::{
    modify_types::string_to_uuid, 
    target_pool::target_account_pool
  }, init::update_embed_data::update_embed_data, models::{
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

#[put("/stores/products")]
pub async fn update_products(
  account_pools: web::Data<AccountPools>,
  payload: web::Json<UpdateProductsPayload>,
  req:HttpRequest
) -> HttpResponse {

  let UpdateProductsPayload {
    store_id, 
    shopify_storefront_store_name, 
    shopify_storefront_access_token,
  } = payload.into_inner();

  let user_id = token_to_string_id(req);

  match user_id {
    Ok(id) => {
        let account_conn = target_account_pool(id.clone(), account_pools).unwrap();
        let store_uuid = string_to_uuid(store_id);

        let store_res = sqlx::query_as::<_, Store>(
          "SELECT * FROM stores WHERE id = $1"
        )
          .bind(store_uuid)
          .fetch_one(&account_conn)
          .await        
          .map_err(|e| {
              eprintln!("Database error: {}", e);
              actix_web::error::ErrorInternalServerError("Error fetching store")
        });

        match store_res {
          Ok(store) => {
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
                    let mut counter: usize = 0;
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
                            return HttpResponse::NotFound().json(serde_json::json!({
                              "status": 404,
                              "message": "Failed to fetch products from Shopify API",
                              "response": []
                            }))                     
                          }).unwrap();

                      let shopify_products: ShopifyProductResponse = response.json()
                          .await
                          .map_err(|e| {
                              eprintln!("JSON parse error: {}", e);
                              return HttpResponse::InternalServerError().json(serde_json::json!({
                                "status": 500,
                                "message": "Failed to unpack products from Shopify",
                                "response": []
                              }))
                          }).unwrap();

                      let number_of_products = shopify_products.data.products.nodes.len();

                      let parsed_products = parse_shopify_products(shopify_products, store_uuid)
                          .map_err(|e| {
                              eprintln!("JSON parse error: {}", e);
                              return HttpResponse::InternalServerError().json(serde_json::json!({
                                "status": 500,
                                "message": "Failed to parse products from Shopify",
                                "response": []
                              }))
                          }).unwrap();

                      let _response = add_products_to_store(
                        account_conn.clone(), 
                        parsed_products.clone(), 
                        store.clone().store_table,
                      )
                        .await
                        .map_err(|err| {
                          println!("Error adding products to '{:?}_products' table: {:?}", store.store_table, err)
                        });


                        let updates_embedder = update_embed_data(account_conn.clone(), store.store_table.clone()).await;

                        match updates_embedder {
                          Ok(_) => {
                            println!("Embedder successfully vectorized {} products", number_of_products.to_string())
                          }

                          Err(err) => {
                            println!("Failed to vectorize products data: {}", err)
                          }
                        }

                        counter += number_of_products;


                        if number_of_products < 250 {
                            println!("{:?}", "No more products to fetch. Ending API fetching loop.");
                            break;
                        } else {
                            println!("{:?}", "Over 250 products fetched, continuing to fetch products.");
                            limit += 250;  
                        }
                    }

                    HttpResponse::Ok().json(serde_json::json!({
                      "status": 200,
                      "message": format!("Successfully updated and vectorized {} products", counter),
                      "response": []
                    }))
                }
                _ => {
                    // Handles unknown platforms
                    println!("Unknown platform: {:?}", store.platform);
                        return HttpResponse::NotFound().json(serde_json::json!({
                          "status": 404,
                          "message": "Platform not found",
                          "response": []
                    }))
                }
              }            

          }

          Err(err) => {
            eprintln!("Error fetching user: {:?}", err);
            return HttpResponse::NotFound().json(serde_json::json!({
              "status": 500,
              "message": "No store exists",
              "response": []
            }))
          }
        }   
    }

    Err(err) => {
      eprintln!("Error fetching user: {:?}", err);
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "status": 401,
        "message": "Token not found or valid",
        "response": []
      }))
    }
  }
}