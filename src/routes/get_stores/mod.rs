
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
  let id_string = token_to_string_id(req);


  match id_string {
    Ok(id) => {


      let account_conn = target_account_pool(id, account_pools);


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
                  "status": "not-found",
                  "message": "Stores not found.",
                  "response": []
                }))
            }).unwrap();


        let stores_res = if stores.len() != 0 {stores} else {Vec::new()};

        HttpResponse::Ok()
        .json(serde_json::json!({
          "status": "success",
          "message": "Successfully fetched stores",
          "response": stores_res
        }))
    }

    Err(err) => {
        eprintln!("Error fetching stores: {:?}", err);
        HttpResponse::Unauthorized().json(serde_json::json!({
        "status": "unauthorized",
        "message": "Invalid or missing token",
        "response": []
      }))
    }
  }

 
}


// THIS HAS NOTHING TO DO WITH FETCHING STORES. IF FETCHES PRODUCTS
// I PUT THIS HERE JUST BECAUSE
// USE THIS TO GET PRODUCTS FROM A PARTICULAR STORE
// SELECT * FROM carmindy_products 
// WHERE store_id = (SELECT id FROM stores WHERE store_table = 'carmindy');