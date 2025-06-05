use actix_web::{web, post, HttpResponse, HttpRequest};
use crate::models::pools_models::AccountPools;
use crate::init::attach_embed_data_checker::attach_embed_data_checker;
use serde::{Deserialize, Serialize};
use crate::helpers::init_account_connection::init_account_connection;

#[derive(Serialize, Deserialize)]
struct Payload {
store_name: String
}

#[post("/stores/products/embed")]
pub async fn embed_table(    
    account_pools: web::Data<AccountPools>,
    payload: web::Json<Payload> ,
    req: HttpRequest
  ) -> HttpResponse {
    let (_account_id, pool) = match init_account_connection(req, account_pools).await {
        Ok(res) => res,
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "error": format!("Invalid token: {:?}", err)
            }));
        }
    };    
    let Payload {
      store_name
    } = payload.into_inner();  

    let _res = attach_embed_data_checker(
      pool.clone(), 
      60 * 60 * 12, 
      store_name.clone())
      .await;

    HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "Message": format!("Embedder has been attached to {}", store_name),
      "response": []
    }))
}
