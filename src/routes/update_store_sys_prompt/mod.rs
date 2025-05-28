use actix_web::{put, web, Result, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{helpers::target_pool::{target_account_pool}, models::pools_models::AccountPools};

#[derive(Serialize, Deserialize)]
struct ReqPayload {
    store_id: Uuid,
    sys_prompt: String
}

#[put("/stores/llm/{account_id}")]
pub async fn update_store_sys_prompt(
    account_pools: web::Data<AccountPools>,
    account_id: web::Path<String>,  
    payload: web::Json<ReqPayload> 
) ->  Result<HttpResponse, Error>{
  let req = payload.into_inner();
  let new_system_prompt = req.sys_prompt;
  let store_id = req.store_id;
  let account_id = account_id.into_inner();

  let account_conn = target_account_pool(account_id, account_pools);

  let result = sqlx::query(
      r#"
          UPDATE stores 
          SET sys_prompt = $1 
          WHERE ID = $2
      "#
  )
    .bind(new_system_prompt)
    .bind(store_id)
    .execute(&account_conn)
    .await
    .map_err(|err| {
        HttpResponse::InternalServerError().json(serde_json::json!({
          "status": "error",
          "message": format!("Internal server error: {}", err),
          "response": []
        }));
        actix_web::error::ErrorInternalServerError(format!("Error updating store system prompt: {}.", err))
    })?;

  if result.rows_affected() == 0 {
      return Ok(HttpResponse::NotFound().json(serde_json::json!({
          "status": "error",
          "message": format!("No store found with ID {}", store_id),
          "response": []
      })));
  }

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": "success",
    "message": "System Prompt for store has been updated",
    "response": []
  })))
}