use sqlx::{Pool, Postgres};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rust_decimal::{prelude::ToPrimitive, Decimal};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CSVValue {
handle: String,
name: String,
description: String,
vendor: String,
category: String,
tags: String,
image: String,
status: String,
price: Decimal,
seo_title: String,
seo_description: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CSV {
  index: i32,
  values: CSVValue,
}

pub async fn upload_csv_to_database(
  account_conn: Pool<Postgres>,
  products: Vec<CSV>,
  table_name: String,
  store_id: Uuid
) -> Result<(), sqlx::error::Error> {
  
  
  if !table_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
      return Err(sqlx::Error::Protocol("Invalid table name".into()));
  }

  let current_time = Utc::now();

  let ids: Vec<i32> = products.iter().map(|p| p.index as i32).collect();
  let names: Vec<String> = products.iter().map(|p| p.values.name.clone()).collect();
  let categories: Vec<String> = products.iter().map(|p| p.values.category.clone()).collect();
  let descriptions: Vec<String> = products.iter().map(|p| p.values.description.clone()).collect();
  let handles: Vec<String> = products.iter().map(|p| p.values.handle.clone()).collect();
  let images: Vec<String> = products.iter().map(|p| p.values.image.clone()).collect();
  let prices: Vec<f64> = products.iter().map(|p| p.values.price.clone().to_f64().unwrap()).collect();
  let seo_descriptions: Vec<String> = products.iter().map(|p| p.values.seo_description.clone()).collect();
  let seo_titles: Vec<String> = products.iter().map(|p| p.values.seo_title.clone()).collect();
  let statuses: Vec<String> = products.iter().map(|p| p.values.status.clone()).collect();
  let tags: Vec<String> = products.iter().map(|p| p.values.tags.clone()).collect();
  let updated_ats: Vec<DateTime<Utc>> = products.iter().map(|_| current_time).clone().collect();
  let vendors: Vec<String> = products.iter().map(|p| p.values.vendor.clone()).collect();
  let created_ats: Vec<DateTime<Utc>> = products.iter().map(|_| current_time.clone()).collect();
  let store_ids: Vec<Uuid> = products.iter().map(|_| store_id.clone()).collect();

  let query_str = format!(
      r#"
      INSERT INTO {}_products (
          id, name, created_at, updated_at, price, vendor, 
          image, handle, description, seo_title, 
          seo_description, category, status, tags, store_id
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
          $15::Uuid[]
      ) ON CONFLICT (id) DO NOTHING
      "#,
      table_name
  );

  let products_added = sqlx::query(&query_str)
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
      .bind(&store_ids)
      .execute(&account_conn)
      .await;

    if let Err(err) = products_added {
          eprintln!("Error adding products into store: {}", err);
          return Err(sqlx::Error::Protocol("Failed to upload products".into()))
    }

    Ok(())
}