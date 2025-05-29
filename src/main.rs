use actix_web::{ HttpServer, Error, Result, App, web };
mod routes;
mod models;
use helpers::init_all_account_connections::init_all_account_connections;
use sqlx::{postgres::PgPoolOptions};
mod init;
use init::{preload_model::preload_model, init_pgai, create_accounts_table::create_accounts_table};
use std::time::Duration;
use routes::{generate_gemma::generate_gemma, 
    create_account::create_account, 
    create_store::create_store,
    update_store_sys_prompt::update_store_sys_prompt,
    get_stores::get_stores,
    delete_store::delete_store,
    login_account::login_account
};
use actix_cors::Cors;
mod helpers;
use models::pools_models::{AdminPool};

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

    for (_account_id, pool) in &account_pools.0 {
        // println!("Adding embedder into {}", account_id);
        init_pgai(pool.clone()).await?;
    }

    let _preloads_model = preload_model(admin_pool.clone())
        .await?;


    println!("===[ Successfully started ]===");

    HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::default()
            // Eventually add CORS
            // .allowed_origin("http://your-nextjs-app.com")
            .allowed_methods(["POST", "DELETE", "GET", "PUT"])
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
        .service(generate_gemma) // POST
        .service(create_account) // POST
        .service(create_store) // POST
        .service(update_store_sys_prompt) // PUT
        .service(get_stores) // GET
        .service(delete_store) // DELETE
        .service(login_account) // POST
    })     
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
