use actix_web::{post, web, HttpResponse};
use serde_json;
use crate::models::generation_models::{DemoPayload};
use crate::models::pools_models::{AccountPools, AdminPool};
mod add_products_suggestion;
use add_products_suggestion::add_products_suggestion;
mod parse_response;
use parse_response::parse_response;
use crate::helpers::target_pool::target_account_pool;

#[post("/generate/demo")]
pub async fn generation_demo (
  payload: web::Json<DemoPayload>,   
  admin_pool: web::Data<AdminPool>,
  account_pools: web::Data<AccountPools>,
) -> HttpResponse  {

  // DID YOU FLUSH THE DATABASE?
  // UPDATE THAT DAMN DEMO_ACCOUNT_ID!!
  // YOU KEEP FORGETTING!!!!

  dotenv::dotenv().ok();
  let payload = payload.into_inner();

  let account_id: String = std::env::var("DEMO_ACCOUNT_ID").expect("No id");

  let pool = match target_account_pool(account_id.clone(), admin_pool, account_pools).await {
      Ok(pool) => pool,
      Err(err) => {
          println!("Failed to connect to user pool: {:?}", err);
          return HttpResponse::NotFound().json(serde_json::json!({
              "status": 404,
              "error": format!("Database connection failed: {}", err)
          }))
      }
  };       
  
  let response_with_products = match add_products_suggestion(
        payload.clone(),
        pool.clone(),
  )
    .await
    {
      Ok(res) => res,
      Err(err) => {
        eprintln!("Error adding products suggestions: {}", err);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": format!("{}", err),
            "response": []
        }))
      }
    };

  let parsed_response = match parse_response(
    response_with_products, 
    pool,
    payload.selector
  )
    .await
    {
      Ok(res) => res,
      Err(err) => {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": format!("{}", err),
            "response": []
        }));
      }
    };

  HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully received message from Server",
      "response": {"data": parsed_response},
  }))
}