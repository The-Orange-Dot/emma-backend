use actix_web::{put, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{auth::token_to_user_id::token_to_string_id, helpers::target_pool::target_account_pool, models::pools_models::AccountPools};

#[derive(Serialize, Deserialize)]
struct ReqPayload {
    store_id: Uuid,
    sys_prompt: String
}

#[put("/stores/emma")]
pub async fn update_store_sys_prompt(
    account_pools: web::Data<AccountPools>,
    payload: web::Json<ReqPayload>,
    req: HttpRequest
) ->  HttpResponse{

  let account_id = token_to_string_id(req);

  match account_id {
    Ok(id) => {
        let ReqPayload {store_id, sys_prompt}  = payload.into_inner();
        let account_conn: sqlx::Pool<sqlx::Postgres> = target_account_pool(id, account_pools);

        let result = sqlx::query(
            r#"
                UPDATE stores 
                SET sys_prompt = $1 
                WHERE ID = $2
            "#
        )
          .bind(sys_prompt)
          .bind(store_id)
          .execute(&account_conn)
          .await
          .map_err(|err| {
              eprintln!("Failed to update store: {}", err);
              HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to update store: {}", err),
                "response": []
              }));
          }).unwrap();    

          if result.rows_affected() == 0 {
              eprintln!("No stores found");
              return HttpResponse::NotFound().json(serde_json::json!({
                  "status": "error",
                  "message": format!("No store found with ID {}", store_id),
                  "response": []
              }));
          }              

          HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "System Prompt for store has been updated",
            "response": []
          }))
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