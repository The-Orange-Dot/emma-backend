use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(FromRow, Deserialize, Serialize)]
pub struct Account {
  pub id: Option<Uuid>,
  pub username: String,
  pub email: Option<String>,
  pub first_name: Option<String>,
  pub last_name: Option<String>,
  pub status: Option<String>,
  pub credits: Option<i32>,
  pub plan: Option<String>,
  pub password: Option<String>,
  pub encryption_nonce: Option<Vec<u8>>,
  pub db_password: Option<String>,
  pub subscription_ends: Option<DateTime<Utc>>,
  pub created_at: Option<DateTime<Utc>>,
  pub updated_at: Option<DateTime<Utc>>,
  pub last_login_at: Option<DateTime<Utc>>
}

#[derive(Serialize, Deserialize)]
pub struct Payload { 
  pub username: String,
  pub password: String,
  pub email: String,
  pub first_name: String,
  pub last_name: String,
}