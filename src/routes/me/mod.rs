use actix_web::{
    get,
    web,
    HttpRequest,
    HttpResponse,
    error::{ErrorUnauthorized, ErrorInternalServerError}
};
use crate::{
  auth::{
    token_to_user_id::token_to_uuid,
  },
  helpers::{
    target_pool::target_admin_pool
  },
  models::{
    pools_models::AdminPool,
    account_models::Account
  }
};
use serde_json::json;

#[get("/me")]
pub async fn me(
    req: HttpRequest,
    admin_pool: web::Data<AdminPool>,
) -> Result<HttpResponse, actix_web::Error> {

    let user_uuid = token_to_uuid(req)?;

    let admin_conn = target_admin_pool(admin_pool);

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
        .bind(user_uuid)
        .fetch_one(&admin_conn)
        .await
        .map_err(|err| {
            eprintln!("Database error fetching user for /me route: {}", err);
            match err {
                sqlx::Error::RowNotFound => {
                    ErrorUnauthorized("Authenticated user not found.")
                },
                _ => ErrorInternalServerError("Failed to retrieve user data."),
            }
        })?;

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "message": "User authenticated",
        "response": {
            "createdAt": account.created_at,
            "credits": account.credits,
            "email": account.email,
            "firstName": account.first_name,
            "lastName": account.last_name,
            "id": account.id,
            "lastLoginAt": account.last_login_at,
            "plan": account.plan,
            "role": account.role,
            "status": account.status,
            "subscriptionEnds": account.subscription_ends,
            "updatedAt": account.updated_at,
            "username": account.username
        }
    })))
}