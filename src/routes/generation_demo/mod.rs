use actix_web::{post, web, HttpResponse};
use serde_json;
use uuid::Uuid;
use crate::models::generation_models::{DemoPayload};
use crate::models::pools_models::{AccountPools, AdminPool};
mod add_products_suggestion;
use add_products_suggestion::add_products_suggestion;
mod parse_response;
use parse_response::parse_response;
use crate::helpers::target_pool::target_account_pool;
use std::net::IpAddr;

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
    pool.clone(),
    payload.clone().selector
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

    let event_data = serde_json::json!({
      "prompt": payload.prompt,
      "selector": payload.selector,
      "response": parsed_response.text,
      "suggested_products": parsed_response.extracted_products
    });

    let parsed_ip_v4: IpAddr = payload.user_ip.parse()
      .map_err(|err| {
        eprintln!("Could not parse user_ip address: {}", err)
      }).unwrap();

    let analytics_uuid = Uuid::new_v4();
    let analytics_query = format!("INSERT INTO {}_analytics (id, event_type, ip_address, user_agent, event_data) VALUES ($1, $2, $3, $4, $5)", &payload.selector);
    let _analytics = sqlx::query(&analytics_query)
    .bind(analytics_uuid)
    .bind("prompt")
    .bind(parsed_ip_v4)
    .bind(payload.user_agent)
    .bind(event_data)
    .execute(&pool)
    .await
    .map_err(|err| {
      eprint!("Error inserting analytics: {}", err);
    })
    ;

  // Insert into store_analytics table
  // id UUID PRIMARY KEY DEFAULT,
  // event_type VARCHAR(50) NOT NULL, -- e.g., 'page_view', 'product_view', 'add_to_cart', 'purchase', 'checkout_started', 'store_visit'
  // event_timestamp TIMESTAMPTZ DEFAULT NOW(), -- Crucial for time-based analysis
  // event_data JSONB, -- Optional: for storing additional, unstructured event-specific data (e.g., { "quantity": 2 }, { "referrer": "google.com" })
  // ip_address INET, -- Optional: for geographical or bot analysis
  // user_agent TEXT, -- Optional: browser/device information  
  // user_data JSONB -- This will use ai to parse user data to categorize based on
        // age_group -> 20s, 30s-40s, 50s and above
        // ethnicity -> european decent, asian decent, latin decent, african decent
        // assumed_gender -> male, female
        // eye_color -> colors
        // hair_color -> colors

  // EVENT TYPES
  // - WHEN A USER SENDS A PROMPT
  // -- event_data -> {prompt: "", response: "", suggested_products: []}
  
  // - WHEN A USER CLICKS ON A PRODUCT (Should have a query tag to mark it)
  // -- event_data -> {product_clicked: product_name}
  

  HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully received message from Server",
      "response": {"data": parsed_response},
  }))
}