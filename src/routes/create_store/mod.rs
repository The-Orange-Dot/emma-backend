use actix_web::{post, HttpResponse, web, Result, Error};
use sqlx::{Postgres, Transaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::helpers::to_snake_case::to_snake_case;
use crate::models::pools_models::{AdminPool, AccountPools};
use crate::helpers::target_pool::{target_admin_pool, target_account_pool};

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

#[post("/stores/{account_id}")]
pub async fn create_store(
    admin_pool: web::Data<AdminPool>,
    account_pools: web::Data<AccountPools>,
    account_id: web::Path<String>,  
    payload: web::Json<Payload> 
  ) -> Result<HttpResponse, Error> {
    let admin_conn = target_admin_pool(admin_pool);
    let account_conn = target_account_pool(account_id.to_string(), account_pools);

    let mut transaction: Transaction<'static, Postgres> = admin_conn.begin()
      .await.expect("Error with transaction for creating store");

    let account_uuid = Uuid::parse_str(&account_id).map_err(|err| {
        actix_web::error::ErrorBadRequest(format!("Invalid UUID: {}", err))
    })?;

    struct AccountRes {
       username: String,
       id: Option<Uuid>,
       status: Option<String>,
       plan: Option<String>,
       db_password: String
    }

    let account = sqlx::query_as!(
        AccountRes,
        r#"
        SELECT id, username, status, plan, db_password FROM public.accounts 
        WHERE id = $1
        "#,
        account_uuid
    )
    .fetch_one(&mut *transaction)
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

    transaction.commit().await.expect("Unable to complete transaction");

    let store_name = &payload.store_name;
    let store_table_name = to_snake_case(store_name);
    let domain = &payload.domain;
    let platform = &payload.platform;

    let _new_store_table_if_not_created = sqlx::query(
          r#"
          CREATE TABLE IF NOT EXISTS stores (
              id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
              account_id UUID NOT NULL,
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
      .execute(&account_conn)
      .await;

    let _new_store = sqlx::query(
      "INSERT INTO stores (account_id, store_name, store_table, domain, platform, sys_prompt) VALUES ($1, $2, $3, $4, $5, $6)"
    )
      .bind(account.id)
      .bind(store_name)
      .bind(store_table_name)
      .bind(domain)
      .bind(platform)
      .bind("")
      .execute(&account_conn)
      .await.expect("TEST");

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