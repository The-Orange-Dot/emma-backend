use actix_web::{put, web, Result, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::pools_models::{AccountPools};

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

// let account_conn = &account_pools.0[account_id.parse::<usize>().unwrap()];

  let account_uuid = Uuid::parse_str(&account_id)
      .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;
  
  // Get the pool
  let account_conn = account_pools.0.get(&account_uuid)
      .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;
          

  println!("test");

  let _results = sqlx::query(
      r#"
          UPDATE stores 
          SET sys_prompt = $1 
          WHERE ID = $2
      "#
  )
    .bind(new_system_prompt)
    .bind(store_id)
    .execute(account_conn)
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