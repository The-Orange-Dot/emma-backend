use actix_web::{web, post, HttpResponse, HttpRequest};
use crate::models::pools_models::AccountPools;
use crate::init::attach_embed_data_checker::attach_embed_data_checker;
use crate::helpers::target_pool::target_account_pool;
use crate::auth::token_to_user_id::token_to_string_id;
use serde::{Deserialize, Serialize};

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
    let account_uuid = token_to_string_id(req);
    let Payload {
      store_name
    } = payload.into_inner();  

    match account_uuid {
      Ok(id) => {
          let account_conn = target_account_pool(id, account_pools).expect("error finding target pool");

          let _res = attach_embed_data_checker(
            account_conn.clone(), 
            60 * 60 * 12, 
            store_name.clone())
            .await;

          HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "Message": format!("Embedder has been attached to {}", store_name),
            "response": []
          }))

      }

      Err(err) => {
        eprint!("Error attaching embedder to {:?}", err);
        HttpResponse::InternalServerError().json(serde_json::json!({
          "status": 500,
          "Message": format!("Failed to attach embedder to {}", store_name),
          "response": []
        }))        
      }
    }

}
