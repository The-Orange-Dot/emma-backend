use actix_web::{web, HttpResponse, post};
use sqlx::{postgres::{PgPoolOptions, PgQueryResult}, Postgres, Transaction, Pool};
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
    
    let mut transaction: Transaction<'static, Postgres> = admin_conn.begin()
        .await
        .expect("Failed to create transaction");

    ensure_store_auth_table(&admin_conn)
    .await
    .expect("Failed to ensure store auth table exists");

    let _create_account_database: Result<PgQueryResult, HttpResponse> = match sqlx::query(&format!("CREATE DATABASE {}", snake_case_name))
        .execute(&admin_conn)
        .await
        {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("Error creating new account's database: {}", err);
                Err(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": 500,
                    "message": "Error creating user account",
                    "response": []
                })))
            }
        };

    // CHANGE THIS DURING PRODUCTION
    let key = get_key().expect("Failed to create key");
    let encrypted_db_password = encrypt_password(&db_password, &key)
        .expect("Failed to encrypt password");

    let _created_accout: Result<PgQueryResult, HttpResponse> = match sqlx::query(&format!(
        "CREATE USER {} WITH PASSWORD '{}' NOINHERIT NOCREATEDB NOCREATEROLE",
        db_username, &db_password
    ))
    .execute(&mut *transaction)
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            eprintln!("Error creating new account: {}", err);
            drop_table(&snake_case_name, &admin_conn).await;
            Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Error creating user account",
                "response": []
            })) )                       
        }
    };

    let _granted_premissions = match sqlx::query(&format!(
        "GRANT ALL PRIVILEGES ON DATABASE {} TO {}",
        snake_case_name, db_username
        ))
        .execute(&mut *transaction)
        .await
        {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("Error granting permission {} to database {}: {}", db_username, snake_case_name, err);
                drop_table(&snake_case_name, &admin_conn).await;                
                Err(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": 500,
                    "message": "Error creating user account",
                    "response": []
                })))                   
            }
        };

    let _granted_ownership = match sqlx::query(&format!(
    "ALTER DATABASE {} OWNER TO {}",
    snake_case_name, db_username
    ))
    .execute(&mut *transaction)
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            eprintln!("Failed to grant ownership to {} to database {}: {}", db_username, snake_case_name, err);
            drop_table(&snake_case_name, &admin_conn).await;            
            Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Error creating user account",
                "response": []
            })))     
        }
    };

    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let hashed_password = Argon2::default()
        .hash_password(dashboard_password.as_bytes(), &salt)
        .expect("Unable to hash password");
    
    let hashed_password = hashed_password.to_string();

    let _add_account_into_accounts_table = match sqlx::query(
        "INSERT INTO accounts (username, email, password, first_name, last_name, db_password) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&snake_case_name)
    .bind(&req.email)
    .bind(hashed_password)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&encrypted_db_password)
    .execute(&mut *transaction)
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            eprintln!("Failed to add user to accounts table: {}", err);
            drop_table(&snake_case_name, &admin_conn).await;            
            Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Error creating user account",
                "response": []
            })))               
        }
    };

    transaction.commit().await.expect("failed to commit transaction");

    let account_id = match sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM accounts WHERE username = $1"
    )
    .bind(&snake_case_name)
    .fetch_one(&admin_conn)
    .await
    {
        Ok(id) => Ok(id.0),
        Err(err) => {
            eprintln!("Failed to get new account id: {}", err);
            drop_table(&snake_case_name, &admin_conn).await;            
            Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": "Error creating user account",
                "response": []
            })))                          
        }
    }.unwrap();


    let account_db_url = match add_account_to_pools(
        &account_pools,
        &admin_url,
        account_id,
        &db_username,
        &encrypted_db_password,
        database_url
    )
        .await
        {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("Failed to add new pool to account pools: {}", err);
                drop_table(&snake_case_name, &admin_conn).await;            
                Err(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": 500,
                    "message": "Error creating user account",
                    "response": []
                })))                  
            }
        }.unwrap();
            
    
   
    let account_conn = match PgPoolOptions::new()
        .max_connections(2)
        .connect(&account_db_url)
        .await
        {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("Failed to connect to new account pool: {}", err);
                drop_table(&snake_case_name, &admin_conn).await;            
                Err(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": 500,
                    "message": "Error creating user account",
                    "response": []
                })))                  
            }
        }.unwrap();

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
            sys_prompt TEXT,
            image BYTEA,
            shopify_storefront_access_token VARCHAR(255),
            shopify_storefront_store_name VARCHAR(255),
            UNIQUE(store_name, store_table, domain)
        )
        "#
    )
    .execute(&account_conn)
    .await
    .map_err(|err| {
        eprintln!("ERROR CREATING STORES TABLE: {:?}", err)
    });

    let _new_store_table_if_not_created = sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS store_products (
                store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
                products_table_name VARCHAR(255) NOT NULL,
                PRIMARY KEY (store_id, products_table_name)
            );
        "#
    )
    .execute(&account_conn)
    .await
    .map_err(|err| {
        eprintln!("ERROR CREATING RELATIONSHIPS TABLE: {:?}", err)
    });

    HttpResponse::Ok().json(serde_json::json!({
        "status": 200,
        "message": "Account created successfully",
        "response": {
            "username": &req.username
        },
    }))
}

async fn drop_database_connection(snake_case_name: &String, admin_conn: &Pool<Postgres>) {
    let _ = sqlx::query(&format!("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'", snake_case_name))
        .execute(admin_conn)
        .await;    
}

async fn drop_table(snake_case_name: &String, admin_conn: &Pool<Postgres>) {
    drop_database_connection(&snake_case_name, &admin_conn).await;
    let _ = sqlx::query(&format!("DROP DATABASE {}", snake_case_name))
        .execute(admin_conn)
        .await;
}

