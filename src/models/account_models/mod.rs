use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(FromRow, Deserialize, Serialize)]
pub struct Account {
  pub id: Uuid,
  pub username: String,
  pub email: String,
  pub first_name: String,
  pub last_name: String,
  pub status: String,
  pub credits: i32,
  pub password: String,
  pub db_password: String,
  pub subscription_ends: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub last_login_at: DateTime<Utc>
}