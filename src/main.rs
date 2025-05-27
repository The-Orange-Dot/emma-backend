use actix_web::{ HttpServer, Error, Result, App, web };
mod routes;
mod models;
use sqlx::{postgres::PgPoolOptions};
mod init_pgai;
use init_pgai::init_pgai;
use std::time::Duration;
use routes::{generate_gemma::generate_gemma, setup_store_database::setup_store_database, create_store::create_store};
use actix_cors::Cors;
use uuid::Uuid;

#[actix_web::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    let pool = PgPoolOptions::new()
        .test_before_acquire(true)
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(2))
        .connect(&database_url)
        .await
        .expect("Error connecting to pool");

    let id = Uuid::new_v4();

    println!("{}", id);

    init_pgai(pool.clone()).await?;

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
        .app_data(web::Data::new(pool.clone()))
        .service(generate_gemma)
        .service(setup_store_database)
        .service(create_store)
    })     
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
