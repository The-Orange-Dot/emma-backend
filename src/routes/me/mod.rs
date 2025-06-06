use actix_web::{get, web, HttpRequest, HttpResponse,};
use crate::{
  auth::{token_to_user_id::token_to_uuid}, 
  helpers::{ 
    target_pool::{target_admin_pool}}, 
    models::{pools_models::AdminPool, 
    account_models::Account
  }
};

#[get("/me")]
pub async fn me(
    req: HttpRequest,
    admin_pool: web::Data<AdminPool>,
) -> HttpResponse {
  
  let user_uuid = match token_to_uuid(req)
  {
    Ok(res) => res,
    Err(err) => {
      eprintln!("Error fetching user: {:?}", err);
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "status": 401,
        "message": "Token not found or valid",
        "response": []
      }));
    }
  };

  let admin_conn = target_admin_pool(admin_pool);

  let account = match sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
      .bind(user_uuid)
      .fetch_one(&admin_conn)
      .await
      {
        Ok(res) => res,
        Err(sqlx::Error::RowNotFound) => {
          return HttpResponse::Unauthorized().json(serde_json::json!({
              "status": 404,
              "message": "No user found",
              "response": []
            }))
        }

        Err(err) => {
          eprintln!("Unauthorized user: {}", err);
          return HttpResponse::Unauthorized().json(serde_json::json!({
              "status": 401,
              "message": "Not authenticated",
              "response": []
          }))
        }
      };

  HttpResponse::Ok().json(serde_json::json!({
    "status": 200,
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