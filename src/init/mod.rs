use actix_web::{Result, HttpResponse, Error};
use serde_json;
pub mod create_accounts_table;
pub mod update_embed_data;
pub mod attach_embed_data_checker;
use attach_embed_data_checker::attach_embed_data_checker;
pub mod preload_model;
use crate::models::store_models::Store;
use std::time::Duration;
use tokio::time;

pub async fn init_pgai (pool: sqlx::Pool<sqlx::Postgres>) -> Result<HttpResponse, Error> {

    let stores = loop {
        match sqlx::query_as::<_, Store>("SELECT * FROM stores")
            .fetch_all(&pool)
            .await
        {
            Ok(stores) => break stores,
            Err(err) => {
                eprint!("Error fetching stores: {}. Retrying in 10 seconds...", err);
                time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
    };

    println!("[Successfully connected to database]");

    for store in &stores {
        let table_name = &store.store_table;

        let _adds_embedder = attach_embed_data_checker(pool.clone(), 60 * 60 * 12, table_name.to_string()).await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": 200,
        "message": "Successfully invoked embedded database from Server"
    })))
}