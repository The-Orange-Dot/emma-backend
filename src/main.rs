use actix_web::{ HttpServer, Error, Result, App, web };
mod routes;
mod models;
use helpers::init_all_account_connections::init_all_account_connections;
use sqlx::{postgres::PgPoolOptions};
mod init_pgai;
use init_pgai::init_pgai;
use std::time::Duration;
use routes::{generate_gemma::generate_gemma, 
    create_account::create_account, 
    create_store::create_store,
    update_store_sys_prompt::update_store_sys_prompt
};
use actix_cors::Cors;
mod helpers;
use models::pools_models::{AdminPool};

#[actix_web::main]
async fn main() -> Result<(), Error> {

    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    let admin_pool = PgPoolOptions::new()
        .test_before_acquire(true)
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(2))
        .connect(&database_url)
        .await
        .expect("Error connecting to pool");

    let account_pools = init_all_account_connections(admin_pool.clone())
        .await?;

    init_pgai(admin_pool.clone()).await?;

    HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::default()
            // Eventually add CORS
            // .allowed_origin("http://your-nextjs-app.com")
            .allowed_methods(["POST"])
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
        .service(generate_gemma)
        .service(create_account)
        .service(create_store)
        .service(update_store_sys_prompt)
    })     
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
