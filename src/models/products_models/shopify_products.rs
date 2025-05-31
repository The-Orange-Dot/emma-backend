use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ShopifyProductResponse {
    pub data: ProductsData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductsData {
    pub products: Products,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Products {
    pub nodes: Vec<Product>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    
    #[serde(rename = "createdAt")]
    pub created_at: String,
    
    pub description: String,

    #[serde(rename = "featuredImage")]
    pub featured_image: Option<FeaturedImage>,
    pub seo: Seo,
    
    #[serde(rename = "priceRange")]
    pub price_range: PriceRange,
    pub vendor: String,
    pub handle: String,
    // pub category: Option<Category>,
    
    #[serde(rename = "publishedAt")]
    pub published_at: DateTime<Utc>,
    
    #[serde(rename = "productType")]
    pub product_type: String,
    
    #[serde(rename = "onlineStoreUrl")]
    pub online_store_url: Option<String>,
    
    #[serde(rename = "availableForSale")]
    pub available_for_sale: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeaturedImage {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Seo {
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceRange {
    #[serde(rename = "minVariantPrice")]
    pub min_variant_price: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Money {
    pub amount: String,
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Category {
//     pub name: String,
// }