use actix_web::{post, HttpResponse, web, Result, Error};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Transaction};
use serde::{Deserialize, Serialize};
use crate::models::account_models::Account;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Params {
  account_id: String
}

#[derive(Serialize, Deserialize)]
struct Payload {
store_name: String,
domain: String,
platform: String,
}

fn to_snake_case(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .replace(' ', "_")
}

#[post("/stores/{account_id}")]
pub async fn create_store(
    admin_pool: actix_web::web::Data<Pool<Postgres>>, 
    account_id: web::Path<String>,  
    payload: web::Json<Payload> 
  ) -> Result<HttpResponse, Error> {
    
    dotenv::dotenv().ok();
    let database_url = std::env::var("POSTGRES_URL")
      .expect("No database url was initialized for the database");

    let mut transaction: Transaction<'static, Postgres> = admin_pool.begin()
      .await.expect("Error with transaction for creating store");

    let account_uuid = Uuid::parse_str(&account_id).map_err(|err| {
        actix_web::error::ErrorBadRequest(format!("Invalid UUID: {}", err))
    })?;

    let account = sqlx::query_as::<_, Account>("SELECT username FROM public.accounts WHERE id = $1")
    .bind(account_uuid)
    .fetch_one(admin_pool.get_ref())
    .await
    .map_err(|err| {
      if let sqlx::Error::RowNotFound = err {
            eprintln!("No account found");
            actix_web::error::ErrorNotFound("Account not found")
        } else {
            eprintln!("Error fetching from 'accounts' table: {}", err);
            actix_web::error::ErrorInternalServerError(format!("Error fetching from accounts table: {}", err))
        }
    })?;

  // Connect to new database to create tables
  let store_db_url = format!(
      "postgres://{}:{}@{}/{}",
      account.username, account.db_password, &database_url, account.username
  );

  let account_pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&store_db_url)
    .await
    .expect("Error connecting to account pool");

  let store_name = &payload.store_name;
  let store_table_name = to_snake_case(store_name);
  let domain = &payload.domain;
  let platform = &payload.platform;

  // Will create the table if it doesn't exist
  let _new_store_table = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS stores (
            id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
            account_id INT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            store_name VARCHAR(255),
            store_table VARCHAR(255),
            domain VARCHAR(255),
            platform VARCHAR(50),
            sys_prompt TEXT
        )
        "#
    )
    .execute(&account_pool)
    .await;

  let _new_store = sqlx::query(
    "INSERT INTO stores (store_name, store_table, domain, platform, sys_prompt) VALUES ($1, $2, $3, $4, $5)"
  )
    .bind(store_name)
    .bind(store_table_name)
    .bind(domain)
    .bind(platform)
    .bind("")
    .execute(&account_pool)
    .await;

  // let _new_products = sqlx::query(
  //     r#"
  //     CREATE TABLE IF NOT EXISTS $1 (
  //         id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  //         store_id uuid REFERENCES store(id) ON DELETE CASCADE,
  //         created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  //         updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  //         handle VARCHAR(255) NOT NULL,
  //         name VARCHAR(255) NOT NULL,
  //         description TEXT,
  //         vendor VARCHAR(255),
  //         price NUMERIC(10,2) CHECK (price >= 0),
  //         status VARCHAR(50) NOT NULL,
  //         embedding vector(768),
  //         UNIQUE(store_id, handle)
  //     )
  //     "#,
  //   )
  //   .bind(store_name)
  //   .execute(&account_pool)
  //   .await;

    

    Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": "success",
      "message": "Store created",
      "response": []
    })))
}