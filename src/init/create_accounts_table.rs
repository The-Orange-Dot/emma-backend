use actix_web::{HttpResponse, Result};
use sqlx::{Pool, Postgres};

pub async fn create_accounts_table(admin_pool: Pool<Postgres>) -> Result<(), HttpResponse> {
      let create_accounts_table = sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
                username TEXT NOT NULL,
                email VARCHAR(255) NOT NULL CHECK (email ~* '^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$'),
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                status VARCHAR(50) DEFAULT 'inactive',
                plan VARCHAR(50),
                credits INT DEFAULT 0,
                role VARCHAR(50) DEFAULT 'user',
                password VARCHAR(255) NOT NULL,
                db_password VARCHAR(255) NOT NULL,
                subscription_ends TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_login_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(username, email)
            );
        "#
    )
        .execute(&admin_pool)
        .await;

  match create_accounts_table {
    Ok(_) => {
        println!("Successfully initialized 'accounts' table in 'postgres' database.");
        Ok(())
    }
    Err(err) => {
        println!("Error initialized 'accounts' table in 'postgres' database: {}", err);
        Err(HttpResponse::BadRequest().json(serde_json::json!({
            "status": 400,
            "Message": format!("Failed to create account: {}", err),
            "response": []
        })))
    }
  }
}