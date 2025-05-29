use sqlx::{Pool, Postgres};
use crate::models::products_models::Product;
use chrono::{NaiveDateTime};

pub async fn add_products_to_store(
  account_conn: Pool<Postgres>,
  products: Vec<Product>,
  table_name: String,
) -> Result<(), sqlx::error::Error> {
  
  if !table_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
      return Err(sqlx::Error::Protocol("Invalid table name".into()));
  }

  let ids: Vec<i32> = products.iter().map(|p| p.id as i32).collect();
  let names: Vec<String> = products.iter().map(|p| p.name.clone()).collect();
  let categories: Vec<String> = products.iter().map(|p| p.category.clone()).collect();
  let descriptions: Vec<String> = products.iter().map(|p| p.description.clone()).collect();
  let handles: Vec<String> = products.iter().map(|p| p.handle.clone()).collect();
  let images: Vec<String> = products.iter().map(|p| p.image.clone()).collect();
  let prices: Vec<f64> = products.iter().map(|p| p.price.clone()).collect();
  let seo_descriptions: Vec<String> = products.iter().map(|p| p.seo_description.clone()).collect();
  let seo_titles: Vec<String> = products.iter().map(|p| p.seo_title.clone()).collect();
  let statuses: Vec<String> = products.iter().map(|p| p.status.clone()).collect();
  let tags: Vec<String> = products.iter().map(|p| p.tags.clone()).collect();
  let updated_ats: Vec<NaiveDateTime> = products.iter().map(|p| p.updated_at.clone()).collect();
  let vendors: Vec<String> = products.iter().map(|p| p.vendor.clone()).collect();
  let created_ats: Vec<NaiveDateTime> = products.iter().map(|p| p.created_at.clone()).collect();

  let query_str = format!(
      r#"
      INSERT INTO {} (
          id, name, created_at, updated_at, price, vendor, 
          image, handle, description, seo_title, 
          seo_description, category, status, tags
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
          $14::text[]
      )
      "#,
      table_name
  );

  sqlx::query(&query_str)
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
      .execute(&account_conn)
      .await
      .map_err(|err| {
          eprintln!("Error adding products into store: {}", err);
          return sqlx::Error::Protocol("Failed to upload products".into())
      });

    Ok(())
}