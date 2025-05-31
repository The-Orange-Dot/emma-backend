use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionToken {
  pub user_id: String,
  pub email: String,
  pub role: String,
  pub expires: usize
}