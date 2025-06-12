use actix_web::{get, web, HttpRequest, HttpResponse, Error};
use crate::{helpers::target_pool::target_admin_pool, models::{pools_models::{AccountPools, AdminPool}, products_models::Product, store_models::{Store, StoreWithProducts}}};
use serde_json;
use crate::helpers::init_account_connection::init_account_connection;
use sqlx::{Pool, Postgres};

#[get("/stores")]
pub async fn get_stores(
    req: HttpRequest,
    admin_pool: web::Data<AdminPool>,
    account_pools: web::Data<AccountPools>,
) -> Result<HttpResponse, Error> {
    let (_account_id, pool) = match init_account_connection(req, admin_pool.clone(), account_pools).await {
        Ok(res) => res,
        Err(err) => {
            println!("Failed to initialize account connection: {:?}", err);
            return Err(actix_web::error::ErrorBadRequest(format!("Invalid token: {:?}", err)));
        }
    };

    let _admin_conn: Pool<Postgres> = target_admin_pool(admin_pool);

    let all_stores = sqlx::query_as::<_, Store>("SELECT * FROM stores")
        .fetch_all(&pool)
        .await
        .map_err(|err| {
            log::error!("Error fetching all stores: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to fetch stores from admin DB")
        })?;    
  
    let mut results: Vec<StoreWithProducts> = Vec::new();

    for store in all_stores {
        let store_name = store.store_name.clone();
        let products_result = fetch_products_for_store_limited(&store.store_table, &pool, 1).await;

        let products_for_this_store = match products_result {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Could not fetch product for store {}: {}. Returning empty products list.", store_name, e);
                Vec::new()
            }
        };

        results.push(StoreWithProducts {
            store, 
            products: products_for_this_store,
        });
    }    

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": 200,
        "message": "Successfully fetched stores with one product each",
        "response": results
    })))

}

pub async fn fetch_products_for_store_limited(
    store_product_table_name: &str,
    store_db_pool: &Pool<Postgres>,
    limit: u32, 
) -> Result<Vec<Product>, Error> {
    let query = format!("SELECT * FROM {} ORDER BY id LIMIT {}", store_product_table_name, limit);

    let products = sqlx::query_as::<_, Product>(&query)
        .fetch_all(store_db_pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch products from table {} with limit {}: {}", store_product_table_name, limit, e);
            actix_web::error::ErrorInternalServerError(format!("Failed to retrieve products for store: {}", e))
        })?;

    Ok(products)
}