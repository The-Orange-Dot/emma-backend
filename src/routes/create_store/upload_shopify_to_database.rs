use sqlx::{PgConnection};
use uuid::Uuid;
use reqwest;
use serde::{Serialize, Deserialize};
use crate::models::products_models::shopify_products::{ShopifyProductResponse, Product};
use sqlx::types::chrono::{DateTime, Utc};
use actix_web::error::{ErrorInternalServerError};

#[derive(Serialize, Deserialize)]
struct GraphQLQuery {
    query: String,
}

#[derive(Serialize, Deserialize)]
struct FeaturedImage {
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Seo {
    description: Option<String>,
    title: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceRange {
    min_variant_price: MinVariantPrice,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinVariantPrice {
    amount: String,
    currency_code: String,
}

#[derive(Serialize, Deserialize)]
struct Category {
    name: String,
}

pub async fn upload_shopify_to_database(
  transaction: &mut PgConnection,
  shopify_storefront_access_token: String,
  shopify_storefront_store_name: String,
  store_uuid: Uuid,
  store_table_name: String
) -> Result<(), actix_web::error::Error> {

  let client = reqwest::Client::new();

  let url = format!("{}/api/unstable/graphql.json", shopify_storefront_store_name);
  let graphql_query_string = r#"{ products(first: 250) { nodes { id title tags updatedAt createdAt description featuredImage { url } seo { description title } priceRange { minVariantPrice { amount currencyCode } } vendor handle category { name } publishedAt productType onlineStoreUrl availableForSale } } }"#;

    let request_body = GraphQLQuery {
        query: graphql_query_string.to_string(),
    };  

  let response = client.post(url)
      .header("Content-Type", "application/json")
      .header("X-Shopify-Storefront-Access-Token", shopify_storefront_access_token)
      .json(&request_body)
      .send()
      .await
      .map_err(|err| {
        eprintln!("Failed to fetch products from Shopify API: {}", err);
        return ErrorInternalServerError(format!("Failed to fetch products from Shopify API: {}", err))
      })?;

  let response_body: ShopifyProductResponse = response.json()
  .await
  .map_err(|err| {
        eprintln!("Failed to parse products from Shopify API: {}", err);
        return ErrorInternalServerError(format!("Failed to parse products from Shopify API: {}", err))
  })?;
  let products: Vec<Product> = response_body.data.products.nodes;

  let current_time = Utc::now();

  let ids_i64: Vec<i64> = products.iter()
      .filter_map(|p| {
          p.id.split('/').last()
              .and_then(|s| s.parse::<i64>().ok())
      })
      .collect();
  let names: Vec<String> = products.iter().map(|p| p.title.clone()).collect();
  let categories: Vec<String> = products.iter().map(|p| p.product_type.clone()).collect();
  let descriptions: Vec<String> = products.iter().map(|p| p.description.clone()).collect();
  let handles: Vec<String> = products.iter().map(|p| p.handle.clone()).collect();
  let images: Vec<String> = products.iter().map(|p| p.featured_image.as_ref().unwrap().url.clone()).collect();
  let prices: Vec<f64> = products.iter().map(|p| p.price_range.min_variant_price.amount.parse::<f64>().unwrap()).collect();
  let seo_descriptions: Vec<String> = products.iter().map(|p| p.seo.description.clone().unwrap_or_default()).collect();
  let seo_titles: Vec<String> = products.iter().map(|p| p.seo.title.clone().unwrap_or_default()).collect();
  let statuses: Vec<String> = products.iter().map(|p| if p.available_for_sale {"active".to_string()} else {"draft".to_string()}).collect();
  let tags: Vec<String> = products.iter().flat_map(|p| p.tags.clone()).collect();
  let updated_ats: Vec<DateTime<Utc>> = products.iter().map(|_| current_time).clone().collect();
  let vendors: Vec<String> = products.iter().map(|p| p.vendor.clone()).collect();
  let created_ats: Vec<DateTime<Utc>> = products.iter().map(|_| current_time.clone()).collect();
  let store_ids: Vec<Uuid> = products.iter().map(|_| store_uuid.clone()).collect();

  let query_str = format!(
        r#"
        INSERT INTO {}_products (
            id, name, created_at, updated_at, price, vendor, 
            image, handle, description, seo_title, 
            seo_description, category, status, tags, store_id
        )
        SELECT * FROM UNNEST(
            $1::bigint[],
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
            $15::Uuid[]
        ) ON CONFLICT (id) DO NOTHING
        "#,
        store_table_name
    );  

  let products_added = sqlx::query(&query_str)
      .bind(&ids_i64)
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
      .bind(&store_ids)
      .execute(transaction)
      .await;

    if let Err(err) = products_added {
          eprintln!("Error adding products into store: {}", err);
          Err(ErrorInternalServerError("Failed to upload products"))
    } else {
          Ok(())
    }

}