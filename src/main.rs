use actix_web::{ HttpServer, Error, Result, App, web };
mod routes;
mod models;
mod middleware;
use helpers::init_all_account_connections::init_all_account_connections;
use sqlx::{postgres::PgPoolOptions};
mod init;
use init::{preload_model::preload_model, init_pgai, create_accounts_table::create_accounts_table};
use std::time::Duration;
use actix_cors::Cors;
mod helpers;
use models::pools_models::{AdminPool};
mod auth;

#[actix_web::main]
async fn main() -> Result<(), Error> {

    dotenv::dotenv().ok();
    let admin_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    let admin_pool = PgPoolOptions::new()
        .test_before_acquire(true)
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(2))
        .connect(&format!("{}/postgres", &admin_url))
        .await
        .expect("Error connecting to pool");

    let _init_accounts_table = create_accounts_table(admin_pool.clone()).await;

    let account_pools = init_all_account_connections(admin_pool.clone(), admin_url.clone())
        .await?;


    for (_account_id, pool) in account_pools.0.read().unwrap().iter() {
        init_pgai(pool.clone()).await?;
}
    let _preloads_model = preload_model(admin_pool.clone())
        .await
        .expect("Couldn't establish connection to LLM Server");

    println!("===[ Successfully started ]===");

    HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::default()
            // Eventually add CORS
            .allowed_origin("http://100.74.191.99:3000")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(["POST", "DELETE", "GET", "PUT"])
            .allow_any_header()
            .supports_credentials()

        )
        .app_data(
            web::PayloadConfig::default()
                .limit(20 * 1024 * 1024) // 20MB upload limit
        )
        .app_data(
            web::JsonConfig::default()
                .limit(20 * 1024 * 1024) // 20MB upload limit
        )
        .app_data(web::Data::new(account_pools.clone()))
        .app_data(web::Data::new(AdminPool(admin_pool.clone())))
        .app_data(web::Data::new(admin_url.clone()))
        .service(routes::generate_gemma::generate_gemma) // POST
        .service(routes::create_account::create_account) // POST
        .service(routes::create_store::create_store) // POST
        .service(routes::update_store_sys_prompt::update_store_sys_prompt) // PUT
        .service(routes::get_stores::get_stores) // GET
        .service(routes::delete_store::delete_store) // DELETE
        .service(routes::login_account::login_account) // POST
        .service(routes::update_products::update_products) // PUT
        .service(routes::logout_account::logout_account) // POST
        .service(routes::me::me) // GET
        .service(routes::get_store_products::get_store_products) // GET
        .service(routes::generation_demo::generation_demo) // POST
        .service(routes::add_products_to_store::add_products_to_store) // POST
        .service(routes::embed_table::embed_table) // POST
    })     
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
