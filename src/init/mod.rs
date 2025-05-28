use actix_web::{Result, HttpResponse, Error};
use serde_json;
pub mod create_accounts_table;

mod update_embed_data;
use update_embed_data::update_embed_data;

mod preload_model;
use preload_model::preload_model;

pub async fn init_pgai (pool: sqlx::Pool<sqlx::Postgres>) -> Result<HttpResponse, Error> {

  let _ = update_embed_data(
      pool.clone(),
      60 * 10 //10 minutes
  )  
      .await;

  let _preloads_model = preload_model(pool.clone())
        .await?;

  Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": "success",
      "message": "Successfully invoked embedded database from Server"
  })))
}