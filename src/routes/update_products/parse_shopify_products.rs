use crate::models::products_models::{Product, shopify_products::ShopifyProductResponse};
use chrono::{DateTime, NaiveDateTime};
use chrono::format::ParseError;

fn parse_shopify_datetime(dt_str: &str) -> Result<NaiveDateTime, ParseError> {
    DateTime::parse_from_rfc3339(dt_str)
        .map(|dt| dt.naive_utc())
}

pub fn parse_shopify_products(response: ShopifyProductResponse) -> Result<Vec<Product>, String> {
    response.data.products.nodes.into_iter().map(|shopify_product| {
        let id_str = shopify_product.id
            .as_str()
            .split('/')
            .last()
            .ok_or_else(|| format!("Invalid ID format: {}", shopify_product.id))?;
        
        let id = id_str.parse::<i64>()
            .map_err(|err| format!("Failed to parse ID {}: {}", id_str, err))?;

        let price = shopify_product.price_range.min_variant_price.amount
            .as_str()
            .parse::<f64>()
            .map_err(|_| format!("Invalid price format: {}", shopify_product.price_range.min_variant_price.amount))?;

        // Handle featured image - more robust version
        let image = match shopify_product.featured_image {
            Some(img) => img.url.clone(),
            None => {
               "".to_string()
            }
        };

        let created_at = parse_shopify_datetime(&shopify_product.created_at)
            .map_err(|e| format!("Invalid created_at format: {}", e))?;

        let updated_at = parse_shopify_datetime(&shopify_product.updated_at)
            .map_err(|e| format!("Invalid created_at format: {}", e))?;


        Ok(Product {
            id,
            name: shopify_product.title,
            created_at,
            updated_at,
            price,
            vendor: shopify_product.vendor,
            image,
            handle: shopify_product.handle,
            description: shopify_product.description,
            seo_title: shopify_product.seo.title.unwrap_or_default(),
            seo_description: shopify_product.seo.description.unwrap_or_default(),
            category: shopify_product.product_type,
            status: if shopify_product.available_for_sale { "active" } else { "inactive" }.to_string(),
            tags: shopify_product.tags.join(","),
        })
    }).collect()
}