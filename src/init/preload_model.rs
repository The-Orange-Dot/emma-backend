use std::time::Duration;

use sqlx::{self, Postgres, Pool};
use actix_web::{Result, Error};


pub async fn preload_model(pool: Pool<Postgres>) -> Result<(), Error> {
    dotenv::dotenv().ok();
    let model = std::env::var("LLM_MODEL").expect("LLM_MODEL not set");

    // Spawn a background task for the infinite loop
    tokio::spawn(async move {
        loop {
            let query = format!(
                "SELECT ai.ollama_generate('{}', '')",
                model
            );

            match sqlx::query(&query).execute(&pool).await {
                Ok(_) => {
                    println!("[PING]: '{}' Model Preloaded", model);
                    tokio::time::sleep(Duration::from_secs(2900)).await;
                }
                Err(err) => {
                    eprintln!("LLM connection error: {}. Retrying in 10s...", err);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    });

    Ok(()) // Return immediately (non-blocking)
}

