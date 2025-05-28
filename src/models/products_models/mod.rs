use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, FromRow)]
pub struct Product {
  pub id: i32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub name: String,
  pub price: f32,
  pub vendor: String,
  pub image: String,
  pub handle: String,
  pub description: String,
  pub seo_title: String,
  pub seo_description: String,
  pub status: String,
  pub category: String,  
  pub tags: String,
  #[sqlx(rename = "type")]
  pub product_type: String
}