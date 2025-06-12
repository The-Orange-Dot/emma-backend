use actix_web::{
  HttpResponse, 
  web, 
  post, 
  Result,
  cookie::{Cookie, time::Duration, SameSite}
};
use serde::{Deserialize, Serialize};
use crate::{
    helpers::target_pool::target_admin_pool, 
    models::{account_models::Account, pools_models::AdminPool}
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use crate::auth;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};

#[derive(Deserialize, Serialize)]
struct LoginPayload {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn login_account(
    payload: web::Json<LoginPayload>,
    admin_pool: web::Data<AdminPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let req = payload.into_inner();
    let admin_conn = target_admin_pool(admin_pool);
    let input_password = &req.password;

    let found_account = match sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE LOWER(email) = LOWER($1)"
    )
    .bind(&req.email.trim()) 
    .fetch_one(&admin_conn)
    .await
    {
        Ok(res) => {
            Ok(res )
        },
        Err(sqlx::Error::RowNotFound) => {
            eprintln!("User with email '{}' not found.", &req.email);
            Err(ErrorUnauthorized("Invalid email or password"))
        },
        Err(err) => {
            eprintln!("Database error while fetching user: {}", err);
           Err(ErrorInternalServerError("Database error during login."))
        }
    }?;

    let stored_hash = match PasswordHash::new(&found_account.password)
        {
            Ok(res) => Ok(res),
            Err(_err) => {
            eprintln!("Incorrect password for account: {}", &req.email);
            Err(ErrorUnauthorized("Invalid email or password"))               
            }
        }?;

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

            let (access_token, refresh_token) = auth::generate_new_tokens(&found_account.id)
                .map_err(|err| {
                    eprintln!("Failed to generate JWT tokens: {:?}", err);
                    ErrorInternalServerError("Failed to create user session")
                })?;

            Ok(HttpResponse::Ok()
                .cookie(
                    Cookie::build("access_token", &access_token)
                        .http_only(true)
                        .secure(true)
                        .same_site(SameSite::None)
                        .path("/login")
                        .domain("meetemma.ai") 
                        .max_age(Duration::minutes(60))
                        .finish()
                )
                .cookie(
                    Cookie::build("refresh_token", &refresh_token)
                        .http_only(true)
                        .secure(true)                       
                        .same_site(SameSite::None)
                        .path("/login")
                        .domain("meetemma.ai")
                        .max_age(Duration::days(30))
                        .finish()
                )
                .json(serde_json::json!({
                "status": 200,
                "message": "Successfully logged in!",
                "response": {"user": account_res}
            })))
        }
        Err(err) => {
            eprintln!("Password verification error: {}", err);
            Err(ErrorUnauthorized("Invalid email or password"))          
        }            
    }
}

