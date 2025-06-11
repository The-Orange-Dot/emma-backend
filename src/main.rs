use actix_web::{HttpServer, App, web};
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
use crate::helpers::start_pool_cleanup_task::start_pool_cleanup_task;

use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::{BufReader, Error as IoError, ErrorKind as IoErrorKind};
use std::fs::File;

// THIS IS NEEDED FOR COOKIES FOR LOCAL DEV -> BE SURE TO USE HTTPS!!
fn load_rustls_config() -> Result<ServerConfig, IoError> {
    let cert_file = &mut BufReader::new(File::open("cert.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);

    let cert_chain: Vec<CertificateDer<'static>> = certs(cert_file)
        .collect::<Result<Vec<CertificateDer<'static>>, IoError>>()?;

    let mut keys: Vec<PrivateKeyDer<'static>> = pkcs8_private_keys(key_file)
        .map(|r| r.map(PrivateKeyDer::Pkcs8))
        .collect::<Result<Vec<PrivateKeyDer<'static>>, IoError>>()?;

    if cert_chain.is_empty() {
        return Err(IoError::new(IoErrorKind::NotFound, "No certificates found in cert.pem"));
    }
    if keys.is_empty() {
        return Err(IoError::new(IoErrorKind::NotFound, "No private keys found in key.pem"));
    }

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .map_err(|e| IoError::new(IoErrorKind::Other, format!("Failed to load TLS config: {}", e)))?;

    Ok(config)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = load_rustls_config();

    dotenv::dotenv().ok();
    println!("Starting initialization...");   

    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("CRASH: {}", panic_info);
    }));

    let admin_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let admin_pool = PgPoolOptions::new()
        .test_before_acquire(true)
        .max_connections(10)
        .idle_timeout(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(10))
        .connect(&format!("{}/postgres", &admin_url))
        .await
        .map_err(|err| std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Failed to connect to admin database: {}", err)
        ))?;

    let cleanup_interval = Duration::from_secs(
        std::env::var("POOL_CLEANUP_INTERVAL_SECS")
            .unwrap_or("300".to_string())
            .parse()
            .unwrap_or(300)
    );

    let idle_timeout = Duration::from_secs(
        std::env::var("POOL_IDLE_TIMEOUT_SECS")
            .unwrap_or("1800".to_string())
            .parse()
            .unwrap_or(1800)
    );

    let account_pools = AccountPools::new();
    start_pool_cleanup_task(account_pools.clone(), cleanup_interval, idle_timeout).await;

    create_accounts_table(admin_pool.clone()).await
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create accounts table: {:?}", e)
        ))?;

    let accounts = sqlx::query_as::<_, AccountInfo>(
        "SELECT id, username, db_password FROM accounts"
    )
    .fetch_all(&admin_pool)
    .await
    .map_err(|e| std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Failed to fetch accounts: {}", e)
    ))?;

    for account in accounts {
        if let Err(e) = account_pools.get_pool(
            account.id,
            &account.username,
            &account.db_password
        ).await {
            eprintln!("Failed to initialize pool for {}: {}", account.username, e);
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
                    .allowed_origin("https://100.74.191.99:3000")
                    .allowed_origin("https://192.168.1.240:3000")
                    .allowed_origin("https://localhost:3000") 
                    .allowed_methods(["POST", "DELETE", "GET", "PUT", "OPTIONS"])
                    .allow_any_header()
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

    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
