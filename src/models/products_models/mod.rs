use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use bigdecimal::BigDecimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod shopify_products;

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
pub struct Product {
  pub id: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub name: String,
  pub price: BigDecimal,
  pub vendor: String,
  pub image: String,
  pub handle: String,
  pub description: String,
  pub seo_title: String,
  pub seo_description: String,
  pub status: String,
  pub category: String,  
  pub tags: String,
  pub store_id: Uuid,
  pub product_url: String
}