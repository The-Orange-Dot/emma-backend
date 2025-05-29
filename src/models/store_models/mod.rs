use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Serialize, Deserialize)]
pub struct StoreCredentials {
    pub db_name: String,
    pub db_username: String,
    pub db_password: String,       // This pw cannot be changed!!
    pub dashboard_username: String,
    pub dashboard_password: String // Initial dashboard password (should be changed later)
}

#[derive(Serialize, Deserialize)]
pub struct Payload { 
  pub store_name: String,
  pub password: String,
  pub email: String,
  pub first_name: String,
  pub last_name: String,
}


#[derive(Serialize, Deserialize, FromRow, Clone, Debug)]
pub struct Store {
  pub id: Uuid,
  pub account_id: Uuid,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub store_name: String,
  pub store_table: String,
  pub domain: String,
  pub platform: String,
  pub sys_prompt: String,
  pub shopify_storefront_store_name: Option<String>,
  pub shopify_storefront_access_token: Option<String>
  
}
