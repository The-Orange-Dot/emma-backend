use actix_web::{put, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::{auth::token_to_user_id::token_to_string_id, helpers::{modify_types::string_to_uuid, target_pool::target_account_pool}, models::pools_models::AccountPools};

#[derive(Serialize, Deserialize)]
struct ReqPayload {
    store_id: String,
    sys_prompt: String
}

#[put("/store/emma")]
pub async fn update_store_sys_prompt(
    account_pools: web::Data<AccountPools>,
    payload: web::Json<ReqPayload>,
    req: HttpRequest
) ->  HttpResponse{

  let account_id = token_to_string_id(req);

  match account_id {
    Ok(id) => {
        let ReqPayload {store_id, sys_prompt}  = payload.into_inner();
        let account_conn: sqlx::Pool<sqlx::Postgres> = target_account_pool(id, account_pools).unwrap();
        let store_uuid = string_to_uuid(store_id.clone());
        let result = sqlx::query(
            r#"
                UPDATE stores 
                SET sys_prompt = $1 
                WHERE ID = $2
            "#
        )
          .bind(sys_prompt)
          .bind(store_uuid.clone())
          .execute(&account_conn)
          .await
          .map_err(|err| {
              eprintln!("Failed to update store: {}", err);
              HttpResponse::InternalServerError().json(serde_json::json!({
                "status": 500,
                "message": format!("Failed to update store: {}", err),
                "response": []
              }));
          }).unwrap();    

          if result.rows_affected() == 0 {
              eprintln!("No stores found");
              return HttpResponse::NotFound().json(serde_json::json!({
                  "status": 500,
                  "message": format!("No store found with ID {}", store_uuid),
                  "response": []
              }));
          }              

          HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "message": "System Prompt for store has been updated",
            "response": []
          }))
    }

    Err(err) => {
      eprintln!("Error fetching user: {:?}", err);
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "status": 401,
        "message": "Token not found or valid",
        "response": []
      }))
    }
  }







}