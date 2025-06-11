use actix_web::{put, web, HttpRequest, HttpResponse,
  error::{ErrorInternalServerError, ErrorBadRequest, ErrorNotFound}
};
use serde::{Deserialize, Serialize};
use crate::{helpers::{modify_types::string_to_uuid}, models::pools_models::AccountPools};
use crate::helpers::init_account_connection::init_account_connection;

#[derive(Serialize, Deserialize)]
struct ReqPayload {
    sys_prompt: String
}

#[put("/store/{store_id}/emma")]
pub async fn update_store_sys_prompt(
    account_pools: web::Data<AccountPools>,
    payload: web::Json<ReqPayload>,
    req: HttpRequest,
    path: web::Path<String>
) ->  Result<HttpResponse, actix_web::Error> {

  let (_account_id, pool) = init_account_connection(req, account_pools)
    .await
    .map_err(|err| {
        ErrorBadRequest(format!("Failed to init account connection: {:?}", err))
    })?;  

  let store_uuid = string_to_uuid(path.into_inner())
    .map_err(|err| {
        ErrorInternalServerError(format!("Failed to parse store id: {:?}", err))
    })?;

  let ReqPayload {sys_prompt}  = payload.into_inner();

  let result = sqlx::query(
      r#"
          UPDATE stores 
          SET sys_prompt = $1 
          WHERE ID = $2
      "#
  )
    .bind(sys_prompt)
    .bind(store_uuid.clone())
    .execute(&pool)
    .await
    .map_err(|err| {
        eprintln!("Failed to update store: {}", err);
        ErrorInternalServerError(format!("Failed to update store: {}", err))
    })?;    

    if result.rows_affected() == 0 {
        eprintln!("No stores found");
        return Err(ErrorNotFound(format!("No store found with ID {}", store_uuid)))
    };              

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": 200,
    "message": "System Prompt for store has been updated",
    "response": []
  })))
}