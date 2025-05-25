use actix_web::{Result, Error, HttpResponse, post, web};
use serde_json;
use sqlx::{Pool, Postgres};
use crate::models::generation_models::{Payload};

mod add_products_suggestion;
use add_products_suggestion::add_products_suggestion;
mod parse_response;
use parse_response::parse_response;


#[post("/generate")]
pub async fn generate_gemma (payload: web::Json<Payload>,pool: web::Data<Pool<Postgres>>) -> Result<HttpResponse, Error>  {
  dotenv::dotenv().ok();
  let req = payload.into_inner();

  // Make this more dynamic in the future
  let target_table = "public.products";

  // println!("[USER PROMPT]: {}", req.prompt);

  let response_with_products = add_products_suggestion(
        req,
        pool.clone(), 
        target_table
  )
    .await
    .expect("Error generating response with product embedding");

  // println!("RESPONSE: {}", response_with_products);

  let parsed_response = parse_response(
    response_with_products, 
    pool.clone())
        .await
        .expect("Error parsing first response to extract text and products.");

  // println!("PARSED MESSAGE: {}", parsed_response.text);

  Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": "success",
      "message": "Successfully received message from Server",
      "response": parsed_response.text,
      "products": parsed_response.products
  })))
}