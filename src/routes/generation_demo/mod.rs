use actix_web::{HttpResponse, post, web};
use serde_json;
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
) -> HttpResponse  {

  dotenv::dotenv().ok();
  let user_id_string: String = std::env::var("DEMO_ACCOUNT_ID").expect("No id");
  let req = payload.into_inner();

  // DID YOU FLUSH THE DATABASE?
  // UPDATE THAT DAMN DEMO_ACCOUNT_ID!!
  // YOU KEEP FORGETTING!!!!

  let pool = match target_account_pool(user_id_string.clone(), account_pools).await {
      Ok(pool) => pool,
      Err(e) => {
          return HttpResponse::NotFound().json(serde_json::json!({
              "status": 404,
              "error": format!("Database connection failed: {}", e)
          }))
      }
  };  
  
  let response_with_products = match add_products_suggestion(
        req.clone(),
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
    req.selector
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

    // println!("{:?}", parsed_response);

  HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully received message from Server",
      "response": {"data": parsed_response},
  }))
}