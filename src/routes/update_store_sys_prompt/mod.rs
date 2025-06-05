use actix_web::{put, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::{helpers::{modify_types::string_to_uuid}, models::pools_models::AccountPools};
use crate::helpers::init_account_connection::init_account_connection;

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

  let (_account_id, pool) = match init_account_connection(req, account_pools).await {
      Ok(res) => res,
      Err(err) => {
          return HttpResponse::BadRequest().json(serde_json::json!({
              "status": 400,
              "error": format!("Invalid token: {:?}", err)
          }));
      }
  }; 

  let ReqPayload {store_id, sys_prompt}  = payload.into_inner();
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
    .execute(&pool)
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