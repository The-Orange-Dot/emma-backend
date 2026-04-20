use actix_web::{HttpServer, App, web, http::header::HeaderValue};
mod routes;
mod models;
use sqlx::postgres::PgPoolOptions;
mod init;
use init::{preload_model::preload_model, init_pgai, create_accounts_table::create_accounts_table};
use std::time::Duration;
use actix_cors::Cors;
mod helpers;
use models::pools_models::{AdminPool, AccountPools};
mod auth;
use crate::models::account_models::AccountInfo;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    println!("Starting initialization...");   

    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("CRASH: {}", panic_info);
    }));

    let admin_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/db".to_string());

    let admin_pool = PgPoolOptions::new()
        .test_before_acquire(true)
        .max_connections(10)
        .idle_timeout(Duration::from_secs(180))
        .acquire_timeout(Duration::from_secs(10))
        .connect(&format!("{}/postgres", &admin_url))
        .await
        .map_err(|err| std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Failed to connect to admin database: {}", err)
        ))?;

    let account_pools: AccountPools = AccountPools::new();

    create_accounts_table(admin_pool.clone()).await
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create accounts table: {:?}", e)
        ))?;

    // FETCHES ALL ACCOUNTS
    let accounts = sqlx::query_as::<_, AccountInfo>(
        "SELECT id, username, db_password FROM accounts"
    )
    .fetch_all(&admin_pool)
    .await
    .map_err(|e| std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Failed to fetch accounts: {}", e)
    ))?;

    // OPENS UP CONNECTION POOL FOR ALL THEIR ACCOUNTS
    for account in accounts {
        if let Err(err) = account_pools.get_pool(
            account.id,
            &account.username,
            &account.db_password
        ).await {
            eprintln!("Failed to initialize pool for {}: {}", account.username, err);
        }
    }

    if let Ok(pools) = account_pools.0.read() {
        for (_account_id, wrapper) in pools.iter() {
            if let Err(e) = init_pgai(wrapper.pool.clone()).await {
                eprintln!("Failed to initialize PGAI: {}", e);
            }
        }
    }

    preload_model(admin_pool.clone()).await
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to preload model: {}", e)
        ))?;

    println!("===[ Successfully started ]===");

    let account_pools_data = web::Data::new(account_pools);
    let admin_pool_data = web::Data::new(AdminPool(admin_pool.clone()));
    let admin_url_data = web::Data::new(admin_url.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("https://meetemma.ai")
                    .allowed_origin("https://localhost:3000") 
                    .allowed_origin("http://localhost:3000") 
                    .allowed_origin("http://100.74.191.99:3000") 
                    .allowed_origin("https://100.74.191.99:3000") 
                    .allowed_origin("https://100.71.24.109:3000") 
                    .allowed_origin("https://100.71.24.109:3000") 
                    .allowed_methods(["POST", "DELETE", "GET", "PUT", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Cookie", "Accept"])
                    .allowed_origin_fn(move |origin: &HeaderValue, _req_head| {
                        if let Ok(origin_str) = origin.to_str() {

                            if origin_str.ends_with(".myshopify.com") {
                                return true;
                            }

                            if origin_str.ends_with(".trycloudflare.com") {
                                return true;
                            }
                            if origin_str.ends_with(".trycloudflare.com/") {
                                return true;
                            }
                        }
                        false
                    })
                    .supports_credentials() 
                    .max_age(3600)
            )
            .app_data(web::PayloadConfig::default().limit(20 * 1024 * 1024))
            .app_data(web::JsonConfig::default().limit(20 * 1024 * 1024))
            .app_data(account_pools_data.clone())
            .app_data(admin_pool_data.clone())
            .app_data(admin_url_data.clone())
            // .service(routes::generate_gemma::generate_gemma)
            // .service(routes::create_account::create_account)
            .service(routes::create_store::create_store)
            .service(routes::update_store_sys_prompt::update_store_sys_prompt)
            .service(routes::get_stores::get_stores)
            .service(routes::delete_store::delete_store)
            .service(routes::login_account::login_account)
            // .service(routes::update_products::update_products)
            .service(routes::logout_account::logout_account)
            .service(routes::me::me)
            .service(routes::get_store_products::get_store_products)
            .service(routes::generation_demo::generation_demo)
            .service(routes::add_products_to_store::add_products_to_store)
            .service(routes::embed_table::embed_table)
            .service(routes::refresh_token::refresh_token)
            .service(routes::health_check::health_check) // GET
            .service(routes::shopify_generate_embeddings::shopify_generate_embeddings) // POST
            .service(routes::generate_text_embedding::generate_text_embedding) // POST
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
