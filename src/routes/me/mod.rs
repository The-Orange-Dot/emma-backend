// src/routes/me.rs
use actix_web::{get, web, HttpRequest, HttpResponse,};
use crate::{auth::validate_jwt, helpers::{modify_types::string_to_uuid, target_pool::{target_admin_pool}}, models::{pools_models::AdminPool, account_models::Account}};

#[get("/me")]
pub async fn me(
    req: HttpRequest,
    admin_pool: web::Data<AdminPool>,
) -> HttpResponse {
    let auth_header = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return HttpResponse::Unauthorized().json("Missing or invalid Authorization header"),
    };

    let claims = match validate_jwt(token) {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid token"),
    };

    let user_id = claims.sub;

    let admin_conn = target_admin_pool(admin_pool);

    let user_uuid = string_to_uuid(user_id.to_string());
    let res = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
        .bind(user_uuid)
        .fetch_one(&admin_conn)
        .await;

    match res {
      Ok(account) => {
        HttpResponse::Ok().json(serde_json::json!({
          "status": "success",
          "message": "User authenticated",
          "response": {
              "created_at": account.created_at,
              "credits": account.credits,
              "email": account.email,
              "first_name": account.first_name,
              "id": account.id,
              "last_login_at": account.last_login_at,
              "plan": account.plan,
              "role": account.role,
              "status": account.status,
              "subscription_ends": account.subscription_ends,
              "updated_at": account.updated_at,
              "username": account.username
          }
        }))
      }

      Err(_) => {
        HttpResponse::Unauthorized().json(serde_json::json!({
          "status": "unauthorized",
          "message": "Not authenticated",
          "response": []
        }))
      }
    }
}