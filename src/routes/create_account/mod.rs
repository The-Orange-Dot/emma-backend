use std::time::Duration;

use actix_web::{web, HttpResponse, post};
use sqlx::postgres::PgPoolOptions;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString};
use argon2::password_hash::rand_core::{OsRng};
use crate::{
    models::account_models::Payload,
    models::pools_models::AccountPools
};
use crate::helpers::{
    generate_random_password::generate_random_password, 
    to_snake_case::to_snake_case,
    target_pool::target_admin_pool,
    add_account_to_pools::add_account_to_pools
};
use crate::auth::password_encoder::{encrypt_password, get_key };

mod ensure_store_auth_table;
use ensure_store_auth_table::ensure_store_auth_table;
use uuid::Uuid;
use crate::models::pools_models::{AdminPool};
use regex::Regex;


#[post("/signup")]
pub async fn create_account(
    admin_pool: web::Data<AdminPool>,
    admin_url: web::Data<String>,
    payload: web::Json<Payload>,
    account_pools: web::Data<AccountPools>,
) -> HttpResponse {
    let req = payload.into_inner();
    let admin_conn = target_admin_pool(admin_pool);
    let admin_url = admin_url.into_inner();
    let new_account_uuid = Uuid::new_v4();

    let _re = match Regex::new(r"^[a-zA-Z0-9_\s]+$")
    {
        Ok(valid) => Ok(valid),
        Err(_err) => {
            Err(HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "message": "Only letters and numbers are accepted",
                "response": []
            })))
        }
    };

    dotenv::dotenv().ok();
    let database_url = std::env::var("POSTGRES_URL")
        .expect("URL for database has not been set for account creation");

    let snake_case_name = to_snake_case(&req.username);
    let db_username = format!("{}", snake_case_name);
    
    let db_password = generate_random_password(24);
    let dashboard_password = req.password;
    
    let mut transaction = match admin_conn.begin().await {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Failed to begin transaction: {}", err);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Failed to initiate database operation.",
                "response": []
            }));
        }
    };
  
    ensure_store_auth_table(&admin_conn)
    .await
    .expect("Failed to ensure store auth table exists");

    let create_account_database= sqlx::query(&format!("CREATE DATABASE {}", snake_case_name))
        .execute(&admin_conn)
        .await;

    if let Err(err) = create_account_database {
        eprintln!("Error creating new account's database: {}", err);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Error creating user account",
            "response": []
        }));
    }

    let key = get_key().expect("Failed to create key");
    let encrypted_db_password = encrypt_password(&db_password, &key)
        .expect("Failed to encrypt password");

    let created_accout = sqlx::query(&format!(
        "CREATE USER {} WITH PASSWORD '{}' NOINHERIT NOCREATEDB NOCREATEROLE",
        db_username, &db_password
    ))
    .execute(&mut *transaction)
    .await;

    if let Err(err) = created_accout {
        eprintln!("Error creating user with password: {}", err);        
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Error creating user account",
            "response": []
        }));
    }

    let granted_premissions = sqlx::query(&format!(
        "GRANT ALL PRIVILEGES ON DATABASE {} TO {}",
        snake_case_name, db_username
        ))
        .execute(&mut *transaction)
        .await;

    if let Err(err) = granted_premissions {
        eprintln!("Error granting privileges for user: {}", err);                
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Error creating user account",
            "response": []
        }));   
    }

    let granted_ownership = sqlx::query(&format!(
    "ALTER DATABASE {} OWNER TO {}",
    snake_case_name, db_username
    ))
    .execute(&mut *transaction)
    .await;

    if let Err(err) = granted_ownership {
        eprintln!("Failed granting ownership to user: {}", err);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Error creating user account",
            "response": []
        }));
    }

    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let hashed_password = Argon2::default()
        .hash_password(dashboard_password.as_bytes(), &salt)
        .expect("Unable to hash password");
    
    let hashed_password = hashed_password.to_string();

    let add_account_into_accounts_table = sqlx::query(
        "INSERT INTO accounts (id, username, email, password, first_name, last_name, db_password) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&new_account_uuid)
    .bind(&snake_case_name)
    .bind(&req.email)
    .bind(hashed_password)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&encrypted_db_password)
    .execute(&mut *transaction)
    .await;

    if let Err(err) = add_account_into_accounts_table {
        eprintln!("Failed to add user to accounts table: {}", err);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Error creating user account",
            "response": []
        }))
    }

    match transaction.commit().await {
        Ok(res) => res,
        Err(err) => {
          eprintln!("Transaction failed on admin side: {}", err); 
          return HttpResponse::InternalServerError().json(serde_json::json!({
              "status": 500,
              "message": "Failed to finalize store deletion.",
              "response": []
          }))          
        }
    } 
          
    // let account_id = match sqlx::query_as::<_, (Uuid,)>(
    //     "SELECT id FROM accounts WHERE username = $1"
    // )
    // .bind(&snake_case_name)
    // .fetch_one(&mut *transaction)
    // .await
    // {
    //     Ok(res) => {
    //         res.0 
    //     },
    //     Err(sqlx::Error::RowNotFound) => {
    //         eprintln!("User with email '{}' not found.", &req.email);
    //         return HttpResponse::Unauthorized().json(serde_json::json!({
    //             "status": 401,
    //             "message": "Invalid email or password",
    //             "response": []
    //         }));
    //     },
    //     Err(err) => {
    //         eprintln!("Database error while fetching user: {}", err);
    //         return HttpResponse::InternalServerError().json(serde_json::json!({
    //             "status": 500,
    //             "message": "Database error during login.",
    //             "response": []
    //         }));
    //     }
    // };

    let account_db_url = match add_account_to_pools(
        &account_pools,
        &admin_url,
        new_account_uuid,
        &db_username,
        &encrypted_db_password,
        database_url
    )
        .await
        {
            Ok(res) => res,
            Err(_) => "Error".to_string()   
        };

    if account_db_url == "Error" {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Error creating user account",
            "response": []
        }));
    };    
    
    // NOW USING ACCOUNT'S POOL RATHER THAN ADMIN'S
    let account_conn = match PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(300))
        .test_before_acquire(true)
        .connect(&account_db_url)
        .await
        {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Failed to connect to new account pool: {}", err);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": 500,
                    "message": "Error creating user account",
                    "response": []
                }));                 
            }
        };

    let mut new_account_transaction = match account_conn.begin().await {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Failed to begin new account's transaction: {}", err);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Failed to initiate database operation.",
                "response": []
            }));
        }
    };              

    let new_store_table_if_not_created = sqlx::query(
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
            sys_prompt TEXT,
            image BYTEA,
            shopify_storefront_access_token VARCHAR(255),
            shopify_storefront_store_name VARCHAR(255),
            UNIQUE(store_name, store_table, domain)
        )
        "#
    )
    .execute(&mut *new_account_transaction)
    .await;

    if let Err(err) = new_store_table_if_not_created {
        eprintln!("Error creating stores table for user {}: {}", db_username, err);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Failed to initiate database operation.",
            "response": []
        }));
    }
    

    let new_products_table_if_not_created = sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS store_products (
                store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
                products_table_name VARCHAR(255) NOT NULL,
                PRIMARY KEY (store_id, products_table_name)
            );
        "#
    )
    .execute(&mut *new_account_transaction)
    .await;

    if let Err(err) = new_products_table_if_not_created {
        eprintln!("Error creating products table for user {}: {}", db_username, err);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": 500,
            "message": "Failed to initiate database operation.",
            "response": []
        }));
    }    

    match new_account_transaction.commit().await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": 200,
                "message": "Account created successfully",
                "response": {
                    "username": &req.username
                },
            }))            
        },
        Err(err) => {
            eprintln!("Failed to create new account: {}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Failed to finalize store deletion.",
                "response": []
            }))            
        }
    } 
}

