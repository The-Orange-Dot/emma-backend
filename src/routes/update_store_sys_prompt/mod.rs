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

  let account_conn = target_account_pool(account_id.to_string(), account_pools);

  let _results = sqlx::query(
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
        actix_web::error::ErrorInternalServerError(format!("Error updating store system prompt: {}.", err))
    })?;

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": "success",
    "message": "System Prompt for store has been updated",
    "response": []
  })))
}