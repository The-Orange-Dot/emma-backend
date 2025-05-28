use actix_web::{Result, HttpResponse, Error};
use serde_json;
pub mod create_accounts_table;
pub mod update_embed_data;
use update_embed_data::update_embed_data;
pub mod preload_model;
use crate::models::store_models::Store;

pub async fn init_pgai (pool: sqlx::Pool<sqlx::Postgres>) -> Result<HttpResponse, Error> {

    let stores = sqlx::query_as::<_,Store>("SELECT * FROM stores")
        .fetch_all(&pool)
        .await
        .expect("Error fetching stores");
    
    for store in &stores {
        let table_name = &store.store_table;

        let _adds_embedder = update_embed_data(pool.clone(), 60 * 60 * 12, table_name.to_string()).await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Successfully invoked embedded database from Server"
    })))
}