use actix_web::{get, web::{self, Query}, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx;
use crate::{
  models::{pools_models::{AccountPools, AdminPool}, store_models::Store}};
use crate::models::products_models::Product;
use crate::helpers::init_account_connection::init_account_connection;


#[derive(Deserialize, Serialize)]
struct RequestQuery {
  store_name: String,
  cursor: Option<i32> 
}

#[get("/store/products")]
pub async fn get_store_products(
  account_pools: web::Data<AccountPools>,
  admin_pool: web::Data<AdminPool>,
  req: HttpRequest
) -> HttpResponse {
    let (_account_id, pool) = match init_account_connection(req.clone(), admin_pool, account_pools).await {
        Ok(res) => res,
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "error": format!("Invalid token: {:?}", err)
            }));
        }
    };  

  let RequestQuery {store_name, cursor} = Query::<RequestQuery>::from_query(req.query_string())
    .unwrap().into_inner();

  let store_res = match sqlx::query_as::<_, Store>("SELECT * FROM stores WHERE store_table = $1")
    .bind(&store_name)
    .fetch_one(&pool)
    .await {
      Ok(res) => res,
      Err(err) => {
        eprint!("Error fetching store: {}", err);
        return HttpResponse::NoContent().json(serde_json::json!({
          "status": 204,
          "message": "Failed to fetch store data",
          "response": []
        }))         
      }
    };

  let products_query = format!(
      "SELECT * FROM {}_products {} ORDER BY id ASC LIMIT 21", 
      &store_name,
      cursor.map(|c| format!("WHERE id > {}", c)).unwrap_or_default()
  );    

  let get_all_products = sqlx::query_as::<_, Product>(&products_query)
    .fetch_all(&pool)
    .await;    

  match get_all_products {
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