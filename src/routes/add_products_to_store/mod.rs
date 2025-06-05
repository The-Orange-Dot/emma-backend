use bigdecimal::BigDecimal;
use sqlx;
use actix_web::{post, web,  HttpRequest, HttpResponse,};
use crate::models::products_models::Product;
use crate::models::pools_models::{AccountPools};
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use crate::helpers::init_account_connection::init_account_connection;

#[derive(Serialize, Deserialize)]
struct Payload {
products: Vec<Product>,
store_name: String
}

#[post("/stores/products")]
pub async fn add_products_to_store(
    account_pools: web::Data<AccountPools>,
    payload: web::Json<Payload>,
    req: HttpRequest,
) -> HttpResponse{
    let (_account_id, pool) = match init_account_connection(req, account_pools).await {
        Ok(res) => res,
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": 400,
                "error": format!("Invalid token: {:?}", err)
            }));
        }
    };

    let Payload {products, store_name} = payload.into_inner();

    let ids: Vec<i32> = products.iter().map(|p| p.id as i32).collect();
    let names: Vec<String> = products.iter().map(|p| p.name.clone()).collect();
    let categories: Vec<String> = products.iter().map(|p| p.category.clone()).collect();
    let descriptions: Vec<String> = products.iter().map(|p| p.description.clone()).collect();
    let handles: Vec<String> = products.iter().map(|p| p.handle.clone()).collect();
    let images: Vec<String> = products.iter().map(|p| p.image.clone()).collect();
    let prices: Vec<BigDecimal> = products.iter().map(|p| p.price.clone()).collect();
    let seo_descriptions: Vec<String> = products.iter().map(|p| p.seo_description.clone()).collect();
    let seo_titles: Vec<String> = products.iter().map(|p| p.seo_title.clone()).collect();
    let statuses: Vec<String> = products.iter().map(|p| p.status.clone()).collect();
    let tags: Vec<String> = products.iter().map(|p| p.tags.clone()).collect();
    let product_urls: Vec<String> = products.iter().map(|p| p.product_url.clone()).collect();
    let updated_ats: Vec<DateTime<Utc>> = products.iter().map(|p| p.updated_at.clone()).collect();
    let vendors: Vec<String> = products.iter().map(|p| p.vendor.clone()).collect();
    let created_ats: Vec<DateTime<Utc>> = products.iter().map(|p| p.created_at.clone()).collect();
    let store_ids: Vec<Uuid> = products.iter().map(|p| p.store_id.clone()).collect();

    let query_str = format!(
        r#"
        INSERT INTO {}_products (
            id, name, created_at, updated_at, price, vendor, 
            image, handle, description, seo_title, seo_description,
            category, status, tags, product_url, store_id
        )
        SELECT * FROM UNNEST(
            $1::integer[],
            $2::text[],
            $3::timestamp[],
            $4::timestamp[],
            $5::float[],
            $6::text[],
            $7::text[],
            $8::text[],
            $9::text[],
            $10::text[],
            $11::text[],
            $12::text[],
            $13::text[],
            $14::text[],
            $15::text[],
            $16::Uuid[]
        ) ON CONFLICT (id) DO NOTHING
        "#,
        store_name
    );

    let _products_added = sqlx::query(&query_str)
      .bind(&ids)
      .bind(&names)
      .bind(&created_ats)
      .bind(&updated_ats)
      .bind(&prices)
      .bind(&vendors)
      .bind(&images)
      .bind(&handles)
      .bind(&descriptions)
      .bind(&seo_titles)
      .bind(&seo_descriptions)
      .bind(&categories)
      .bind(&statuses)
      .bind(&tags)
      .bind(&product_urls)
      .bind(&store_ids)
      .execute(&pool)
      .await
      .map_err(|err| {
          eprintln!("Error adding products into store: {}", err);
          return sqlx::Error::Protocol("Failed to upload products".into())
      });

    HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "Message": "Added products to store",
      "response": []
    }))
}