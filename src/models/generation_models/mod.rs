use serde::{Serialize, Deserialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Product {
    pub name: String,
    pub description: String,
    pub price: String,  // Change to f64 for prices for production
    pub image: String,
    pub handle: String,
    pub vendor: String,
}

#[derive(Deserialize)]
pub struct Payload {
    pub prompt: String,
    pub images: Vec<ImageData>, 
}

#[derive(Deserialize)]
pub struct ImageData {
    pub data_url: String,
}

#[derive(Serialize, Deserialize)] 
pub struct GemmaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub images: Vec<String>,
}

#[derive(Serialize)]
pub struct ParsedResponse {
  pub text: String,
  pub products: Value
}