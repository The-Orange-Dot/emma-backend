use sqlx::{self, Postgres, Pool};
use actix_web::{Result, Error};

pub async fn preload_model (pool: Pool<Postgres>) -> Result<(), Error> {
dotenv::dotenv().ok();

let model = std::env::var("LLM_MODEL")
.expect("Failed to initialize model for preload.");

let query = format!(
  "SELECT ai.ollama_generate(
      '{}',
      ''
  )", model
);

let _preloads_model = sqlx::query(
  &query
).execute(&pool)
.await
.map_err(|err| {
  eprint!("Failed to preload model '{}': {}", model, err);
  println!("");
  actix_web::error::ErrorInternalServerError(format!("Failed to preload model: {}", err ))
});

println!("'{}' has been preloaded.", model);
Ok(())
}