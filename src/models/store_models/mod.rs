use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

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


#[derive(Serialize, Deserialize)]
pub struct Store {
  pub id: String,
  pub store_id: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub handle: String,
  pub name: String,
  pub description: String,
  pub vendor: String,
  pub price: f32,
  pub status: String
}
