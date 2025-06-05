use actix_web::{get, web::{self, Query}, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx;
use crate::{
  models::{pools_models::AccountPools, store_models::Store}};
use crate::models::products_models::Product;
use crate::helpers::init_account_connection::init_account_connection;


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
    let (_account_id, pool) = match init_account_connection(req.clone(), account_pools).await {
        Ok(res) => res,
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "error": format!("Invalid token: {:?}", err)
            }));
        }
    };  

  let query = Query::<RequestQuery>::from_query(req.query_string())
    .unwrap().into_inner();

  let RequestQuery {store_name, cursor} = query;

  let products_query = format!(
      "SELECT * FROM {}_products {} ORDER BY id ASC LIMIT 21", 
      &store_name,
      cursor.map(|c| format!("WHERE id > {}", c)).unwrap_or_default()
  );
  let res = sqlx::query_as::<_, Product>(&products_query)
    .fetch_all(&pool)
    .await;

  let store_query: String = format!("SELECT * FROM stores WHERE store_table = '{}'", store_name);
  let store_res = sqlx::query_as::<_, Store>(&store_query)
    .fetch_one(&pool)
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