use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use chrono::{DateTime, Utc, NaiveDateTime};
use uuid::Uuid;

pub mod shopify_products;

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
pub struct Product {
  pub id: i64,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
  pub name: String,
  pub price: f64,
  pub vendor: String,
  pub image: String,
  pub handle: String,
  pub description: String,
  pub seo_title: String,
  pub seo_description: String,
  pub status: String,
  pub category: String,  
  pub tags: String,
  pub store_id: Uuid
}