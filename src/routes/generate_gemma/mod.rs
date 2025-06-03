use actix_web::{HttpResponse, post, web};
use serde_json;
use sqlx::{Pool, Postgres};
use crate::models::generation_models::{Payload};

mod add_products_suggestion;
use add_products_suggestion::add_products_suggestion;
mod parse_response;
use parse_response::parse_response;


#[post("/generate")]
pub async fn generate_gemma (payload: web::Json<Payload>, pool: web::Data<Pool<Postgres>>) -> HttpResponse  {
  dotenv::dotenv().ok();
  let req = payload.into_inner();

  // Make this more dynamic in the future
  let target_table = "public.products";


  println!("[USER PROMPT]: {}", req.prompt);

  let response_with_products = add_products_suggestion(
        req,
        pool.clone(), 
        target_table
  )
    .await
    .map_err(|err| {
        eprint!("Error parsing products in response: {:?}", err);
        HttpResponse::InternalServerError().json(serde_json::json!({
          "status": 500,
          "message": format!("Error parsing product in response: {:?}", err),
          "response": []
        }))
    }).unwrap();
    
  // println!("RESPONSE: {}", response_with_products);

  let parsed_response = parse_response(
    response_with_products, 
    pool.clone())
        .await
        .map_err(|err| {
            HttpResponse::InternalServerError().json(serde_json::json!({
              "status": 500,
              "message": format!("Error parsing response from Emma: {}", err),
              "response": []
            }))
        }).unwrap();
  // println!("PARSED MESSAGE: {}", parsed_response.text);

  HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully received message from Server",
      "response": parsed_response.text,
      "products": parsed_response.products
  }))
}