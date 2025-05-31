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
  
    let user_uuid = token_to_uuid(req);

    match user_uuid {
      Ok(id) => {
          let admin_conn = target_admin_pool(admin_pool);

          let res = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
              .bind(id)
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

            Err(err) => {
              eprintln!("Error fetching user: {:?}", err);
              HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "unauthorized",
                "message": "Not authenticated",
                "response": []
              }))
            }
          }              
      }
      Err(err) => {
        eprintln!("Error fetching user: {:?}", err);
        return HttpResponse::Unauthorized().json(serde_json::json!({
          "status": "unauthorized",
          "message": "Token not found or valid",
          "response": []
        }))
      }
    }
}