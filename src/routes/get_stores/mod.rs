
use crate::helpers::target_pool::{target_account_pool};
use actix_web::{get, web, HttpRequest, HttpResponse};
use crate::models::{pools_models::AccountPools, store_models::Store};
use crate::auth::token_to_user_id::token_to_string_id;
use serde_json;

#[get("/stores")]
pub async fn get_stores(
  account_pools: web::Data<AccountPools>,
  req: HttpRequest,
) -> HttpResponse{
  let id_string: Result<String, HttpResponse> = token_to_string_id(req);


  match id_string {
    Ok(id) => {
      let account_conn: sqlx::Pool<sqlx::Postgres> = target_account_pool(id, account_pools).unwrap();


        let stores = sqlx::query_as::<_, Store>("
              SELECT id, created_at, account_id,
              updated_at, store_name, 
              store_table, domain, 
              platform, sys_prompt, 
              shopify_storefront_access_token, 
              shopify_storefront_store_name,
              image
              from stores
            ")
            .fetch_all(&account_conn)
            .await
            .map_err(|err| {
                println!("Error fetching stores: {}", err);
                HttpResponse::NotFound().json(serde_json::json!({
                  "status": 404,
                  "message": "Stores not found.",
                  "response": []
                }))
            }).unwrap();


        let stores_res = if stores.len() != 0 {stores} else {Vec::new()};

        HttpResponse::Ok()
        .json(serde_json::json!({
          "status": 200,
          "message": "Successfully fetched stores",
          "response": stores_res
        }))
    }

    Err(err) => {
        eprintln!("Error fetching stores: {:?}", err);
        HttpResponse::Unauthorized().json(serde_json::json!({
        "status": 401,
        "message": "Invalid or missing token",
        "response": []
      }))
    }
  }

 
}
