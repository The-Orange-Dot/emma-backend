use actix_web::{web, HttpResponse, post};
use sqlx::{postgres::{PgPoolOptions}, Postgres, Transaction};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString};
use rand_core::OsRng;
use std::error::Error;
use crate::{
    models::account_models::Payload,
};
use crate::helpers::{
    generate_random_password::generate_random_password, 
    to_snake_case::to_snake_case,
    get_account_psql_link::get_account_psql_link,
    target_pool::target_admin_pool,
    install_extensions::install_extensions
};
use password_encoder::{encrypt_password, get_or_create_dev_key};

mod ensure_store_auth_table;
use ensure_store_auth_table::ensure_store_auth_table;

use crate::models::pools_models::{AdminPool};

#[post("/signup")]
pub async fn create_account(
    admin_pool: web::Data<AdminPool>,
    admin_url: web::Data<String>,
    payload: web::Json<Payload>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let req = payload.into_inner();
    let admin_conn = target_admin_pool(admin_pool);
    let admin_url = admin_url.into_inner();

    dotenv::dotenv().ok();
    let database_url = std::env::var("POSTGRES_URL")
        .expect("URL for database has not been set for account creation");

    let snake_case_name = to_snake_case(&req.username);
    let db_username = format!("{}", snake_case_name);
    let dashboard_username = format!("user_{}", snake_case_name);
    
    let db_password = generate_random_password(24);
    let dashboard_password = req.password;
    
    let mut transaction: Transaction<'static, Postgres> = admin_conn.begin().await?;

    ensure_store_auth_table(&admin_conn)
    .await?;


    sqlx::query(&format!("CREATE DATABASE {}", snake_case_name))
        .execute(&admin_conn)
        .await?;

    // CHANGE THIS DURING PRODUCTION
    let key = get_or_create_dev_key()?;
    let encrypted_db_password = encrypt_password(&db_password, &key)
        .expect("Failed to encrypt password");

    sqlx::query(&format!(
        "CREATE USER {} WITH PASSWORD '{}' NOINHERIT NOCREATEDB NOCREATEROLE",
        db_username, &db_password
    ))
    .execute(&mut *transaction)
    .await?;

    sqlx::query(&format!(
    "GRANT ALL PRIVILEGES ON DATABASE {} TO {}",
    snake_case_name, db_username
    ))
    .execute(&mut *transaction)
    .await?;

    sqlx::query(&format!(
    "ALTER DATABASE {} OWNER TO {}",
    snake_case_name, db_username
    ))
    .execute(&mut *transaction)
    .await?;

    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let hashed_password = Argon2::default()
        .hash_password(dashboard_password.as_bytes(), &salt)?
        .to_string();

    sqlx::query(
        "INSERT INTO accounts (username, email, password, first_name, last_name, db_password) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&snake_case_name)
    .bind(&req.email)
    .bind(hashed_password)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&encrypted_db_password)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    
    let store_db_url = get_account_psql_link(db_username, encrypted_db_password, database_url);
   
    let _store_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&store_db_url)
        .await?;

    let _ = install_extensions(&admin_url, &snake_case_name)
        .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "db_name ": snake_case_name,
        "dashboard_username ": dashboard_username,
        "dashboard_password ": dashboard_password,
    })))
}

