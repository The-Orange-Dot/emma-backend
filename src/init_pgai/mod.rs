use actix_web::{Result, HttpResponse, Error};
use chrono::NaiveDateTime;
use serde_json;
use sqlx::Row;

mod update_embed_data;
use update_embed_data::update_embed_data;

// mod create_extensions;
// use create_extensions::create_extensions;

// mod create_table;
// use create_table::create_table;

pub async fn init_pgai (pool: sqlx::Pool<sqlx::Postgres>) -> Result<HttpResponse, Error> {
  
  // CREATE SCRIPT FOR FORMATTING PRICE TO FLOAT RATHER THAN TEXT
  // let _ = create_table(
  //     pool.clone(), 
  //     "products", 
  //     &["name TEXT", "category TEXT", "description TEXT", "price TEXT", "specs TEXT", "image TEXT"])
  //     .await;
    
  // let _ = create_extensions(
  //     pool.clone(), 
  //     &["ai", "vector", "vectorscale"])
  //     .await;

  // Will run in the background updating db every x minutes
  let _ = update_embed_data(
      pool.clone(),
      60 * 10 //10 minutes
  )  
      .await;

  let test = sqlx::query(r#"
      SELECT * FROM public.products WHERE id = 267
  "#)
  .fetch_optional(&pool)  // Returns Result<Option<PgRow>, Error>
  .await
  .map_err(|e| {
      eprintln!("Query failed: {:?}", e);
      actix_web::error::ErrorInternalServerError("Database error")
  })?;


  match test {
      Some(row) => {
          println!("=== DEBUG PRODUCT ===");
          println!("[ Name ]: {:?}", row.try_get::<String, _>("name")); 
          println!("");
          println!("[ Price ]: {:?}", row.try_get::<String, _>("price"));
          println!("");
          println!("[ created_at ]: {:?}", row.try_get::<NaiveDateTime, _>("created_at"));
          println!("");
          println!("[ updated_at ]: {:?}", row.try_get::<NaiveDateTime, _>("updated_at"));

      },
      None => println!("No product found with ID 1"),
  }

  Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": "success",
      "message": "Successfully invoked embedded database from Server"
  })))
}