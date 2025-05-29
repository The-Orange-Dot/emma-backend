use actix_web::{Error, HttpResponse, web, post};
use serde::{Deserialize, Serialize};
use crate::{
    helpers::target_pool::target_admin_pool, 
    models::{account_models::Account, pools_models::AdminPool}
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};

#[derive(Deserialize, Serialize)]
struct LoginPayload {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn login_account(
    payload: web::Json<LoginPayload>,
    admin_pool: web::Data<AdminPool>,
) -> Result<HttpResponse, Error> {
    let req = payload.into_inner();
    let admin_conn = target_admin_pool(admin_pool);
    let input_password = &req.password;

    eprintln!("Attempting login with email: '{}'", &req.email);

    let found_account = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE LOWER(email) = LOWER($1)"
    )
    .bind(&req.email.trim()) 
    .fetch_one(&admin_conn)
    .await
    .map_err(|err| {
        eprintln!("Database error: {}", err);
        actix_web::error::ErrorUnauthorized("Invalid email or password")
    })?;

    let stored_hash = PasswordHash::new(&found_account.password)
        .map_err(|err| {
            eprintln!("Failed to parse stored hash: {}", err);
            actix_web::error::ErrorInternalServerError(format!("Internal server error: {}", err))
        })?;

    Argon2::default().verify_password(input_password.as_bytes(), &stored_hash)
        .map_err(|err| {
            eprintln!("Failed to verify password: {}", err);
            actix_web::error::ErrorUnauthorized("Invalid email or password")
        })?;

    let account_res = serde_json::json!({
      "created_at": found_account.created_at,
      "credits": found_account.credits,
      "email": found_account.email,
      "first_name": found_account.first_name,
      "id": found_account.id,
      "last_login_at": found_account.last_login_at,
      "plan": found_account.plan,
      "status": found_account.status,
      "subscription_ends": found_account.subscription_ends,
      "updated_at": found_account.updated_at,
      "username": found_account.username
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Successfully logged in!",
        "account": account_res,
    })))
}