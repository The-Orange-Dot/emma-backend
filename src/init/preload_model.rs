use std::time::Duration;

use sqlx::{self, Postgres, Pool};
use actix_web::{Result, Error};
use tokio::time;


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

  let _preloads_modes = loop {
    match sqlx::query(
    &query
  )
    .execute(&pool)
    .await
    {
      Ok(_res) => {
        println!("[PING]: '{}' Model Preloaded", model);
        time::sleep(Duration::from_secs(2900)).await;
        continue;
      },
      Err(err) => {
        eprint!("Error establishing connection to LLM server: {}", err);
        eprintln!("Retrying connection in 10 seconds...");
        time::sleep(Duration::from_secs(10)).await;
        continue;
      }
    }
  };

}