use actix_web::{get, web::{self, Query}, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx;
use crate::{
  auth::token_to_user_id::token_to_string_id, 
  helpers::target_pool::target_account_pool, 
  models::{pools_models::AccountPools, store_models::Store}};
use crate::models::products_models::Product;


#[derive(Deserialize, Serialize)]
struct RequestQuery {
  store_name: String,
  cursor: Option<i32> 
}

#[get("/store")]
pub async fn get_store_products(
  account_pools: web::Data<AccountPools>,
  req: HttpRequest
) -> HttpResponse {
  let id_string = token_to_string_id(req.clone());
  
  match id_string {
    Ok(id) => {
      let account_conn = target_account_pool(id, account_pools).unwrap();
      let query = Query::<RequestQuery>::from_query(req.query_string())
        .unwrap().into_inner();

      let RequestQuery {store_name, cursor} = query;

      let products_query = format!(
          "SELECT * FROM {}_products {} ORDER BY id ASC LIMIT 21", 
          &store_name,
          cursor.map(|c| format!("WHERE id > {}", c)).unwrap_or_default()
      );
      let res = sqlx::query_as::<_, Product>(&products_query)
        .fetch_all(&account_conn)
        .await;

      let store_query: String = format!("SELECT * FROM stores WHERE store_table = '{}'", store_name);
      let store_res = sqlx::query_as::<_, Store>(&store_query)
        .fetch_one(&account_conn)
        .await
        .map_err(|err| {
          eprint!("Error fetching store: {}", err);
          HttpResponse::NoContent().json(serde_json::json!({
            "status": 204,
            "message": "Failed to fetch store data",
            "response": []
          })) 
        }).unwrap();

      match res {
        Ok(mut data) => {
          let has_more = data.len() > 20;
          if has_more {
            data.pop();
          }
          
          let next_cursor = data.last().map(|p| p.id);
          
          HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "message": "Successfully fetched products and store",
            "response": {
              "store": store_res, 
              "products": data,
              "next_cursor": next_cursor,
              "has_more": has_more
            }
          }))              
        }

        Err(err) => {
          eprint!("No products have been found: {}", err);
          HttpResponse::NoContent().json(serde_json::json!({
            "status": 204,
            "message": "No products have been found",
            "response": []
          }))          
        }
      }
    }

    Err(_) => {
      HttpResponse::Unauthorized().json(serde_json::json!({
        "status": 401,
        "message": "Unauthorized Access",
        "response": []
      }))
    }
  }
}