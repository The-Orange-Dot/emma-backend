use actix_web::{
  HttpResponse, 
  web, 
  post, 
  cookie::{Cookie, time::Duration, SameSite}
};
use serde::{Deserialize, Serialize};
use crate::{
    helpers::target_pool::target_admin_pool, 
    models::{account_models::Account, pools_models::AdminPool}
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use crate::auth;

#[derive(Deserialize, Serialize)]
struct LoginPayload {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn login_account(
    payload: web::Json<LoginPayload>,
    admin_pool: web::Data<AdminPool>,
) -> HttpResponse {
    let req = payload.into_inner();
    let admin_conn = target_admin_pool(admin_pool);
    let input_password = &req.password;

    // eprintln!("Attempting login with email: '{}'", &req.email);

    let found_account = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE LOWER(email) = LOWER($1)"
    )
    .bind(&req.email.trim()) 
    .fetch_one(&admin_conn)
    .await
    .map_err(|err| {
        eprintln!("Failed to find existing user: {}", err);
        HttpResponse::Unauthorized().json(serde_json::json!({
          "status": 401,
          "message": "Invalid email or password",
          "response": []
        }))
    }).unwrap();

    let stored_hash = PasswordHash::new(&found_account.password)
        .map_err(|err| {
            eprintln!("Failed to parse stored hash: {}", err);
            HttpResponse::Unauthorized().json(serde_json::json!({
              "status": 401,
              "message": "Invalid email or password",
              "response": []
            }))
        }).unwrap();

    let password_verification = Argon2::default().verify_password(input_password.as_bytes(), &stored_hash);

    match password_verification {
        Ok(_) => {
            let account_res = serde_json::json!({
              "created_at": found_account.created_at,
              "credits": found_account.credits,
              "email": found_account.email,
              "first_name": found_account.first_name,
              "id": found_account.id,
              "last_login_at": found_account.last_login_at,
              "plan": found_account.plan,
              "role": found_account.role,
              "status": found_account.status,
              "subscription_ends": found_account.subscription_ends,
              "updated_at": found_account.updated_at,
              "username": found_account.username
            });

            let token = auth::create_jwt(&found_account.id.to_string())
                .expect("failed to create JWT token");

            HttpResponse::Ok()
                .cookie(
                    Cookie::build("jwt", &token)
                        .http_only(true)
                        // .secure(true)
                        .same_site(SameSite::Lax)
                        .path("/")
                        .max_age(Duration::days(30))
                        .finish()
                )
                .json(serde_json::json!({
                  "status": 200,
                  "message": "Successfully logged in!",
                  "token": token,
                  "response": {"user": account_res}
              
            }))
        }
        Err(err) => {
            let (status_code, error_message) = match err {
                argon2::password_hash::Error::Password => {
                    (actix_web::http::StatusCode::UNAUTHORIZED, "Invalid password or email address.")
                }
                _ => {
                    eprintln!("Password verification error: {}", err);
                    (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error during authentication.")
                }
            };

            HttpResponse::build(status_code).json(serde_json::json!({
              "status": status_code.as_str().to_lowercase(),
              "message": error_message,
              "response": []
            }))
        }
    }
}