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

  let preloads_model = sqlx::query(
    &query
  ).execute(&pool)
  .await;

  match preloads_model {
    Ok(_) => {
        println!("'{}' has been preloaded.", model);
    }
    Err(err) => {
        eprint!("'{}' could not be preloaded: {}", model, err);
        actix_web::error::ErrorInternalServerError(format!("Failed to preload model: {}", err ));
    }
  }

  Ok(())
}