use actix_web::{Result, Error, HttpResponse, post, web};
use serde_json;
use sqlx::{Pool, Postgres};
use crate::models::generation_models::{DemoPayload};
use crate::models::pools_models::AccountPools;
mod add_products_suggestion;
use add_products_suggestion::add_products_suggestion;
mod parse_response;
use parse_response::parse_response;
use crate::helpers::target_pool::target_account_pool;

#[post("/generate/demo")]
pub async fn generation_demo (
  payload: web::Json<DemoPayload>,   
  account_pools: web::Data<AccountPools>,
) -> Result<HttpResponse, Error>  {
 
  dotenv::dotenv().ok();
  let user_id_string = std::env::var("DEMO_ACCOUNT_ID").expect("No id");
  let req = payload.into_inner();

  let account_conn: Pool<Postgres> = target_account_pool(user_id_string, account_pools).unwrap();

  let response_with_products = add_products_suggestion(
        req.clone(),
        account_conn.clone(), 
  )
    .await
    .expect("Error generating response with product embedding");

  let parsed_response = parse_response(
    response_with_products, 
    account_conn,
    req.selector
  
  )
    .await
    .expect("Error parsing first response to extract text and products.");


  Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully received message from Server",
      "response": parsed_response.text,
      "products": parsed_response.products
  })))
}