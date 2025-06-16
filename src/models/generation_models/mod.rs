use serde::{Serialize, Deserialize};
use serde_json::Value;
use sqlx::FromRow;
use bigdecimal::BigDecimal;


#[derive(Debug, Serialize, FromRow)]
pub struct Product {
    pub name: String,
    pub description: String,
    pub price: BigDecimal, 
    pub image: String,
    pub handle: String,
    pub vendor: String,
    pub product_url: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoPayload {
    pub prompt: String,
    pub images: Vec<ImageData>, 
    pub selector: String,
    pub user_ip: String,
    pub user_agent: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Debug)]
pub struct ParsedResponse {
  pub text: String,
  pub products: Value,
  pub store_domain: String,
  pub extracted_products: String

}