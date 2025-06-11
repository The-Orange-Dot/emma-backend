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

// use std::io::BufReader;
// use std::fs::File;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    println!("Starting initialization...");   

    // rustls::crypto::aws_lc_rs::default_provider()
    //     .install_default()
    //     .unwrap();

    // let (cert_path, key_path) = if std::env::var("APP_ENV") == Ok("development".to_string()) {
    //     println!("Working with dev-cert and dev-key");
    //     ("dev-cert.pem", "dev-key.pem")
    // } else {
    //     ("cert.pem", "key.pem")
    // };

    // let mut certs_file = BufReader::new(File::open(cert_path).unwrap());
    // let mut key_file = BufReader::new(File::open(key_path).unwrap());

    // // load TLS certs and key
    // // to create a self-signed temporary cert for testing:
    // // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    // let tls_certs = rustls_pemfile::certs(&mut certs_file)
    //     .collect::<Result<Vec<_>, _>>()
    //     .unwrap();
    // let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
    //     .next()
    //     .unwrap()
    //     .unwrap();

    // let tls_config = rustls::ServerConfig::builder()
    //     .with_no_client_auth()
    //     .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
    //     .unwrap();

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
    // .bind_rustls_0_23(("0.0.0.0", 8080), tls_config)?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
