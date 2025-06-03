use std::collections::HashSet;
use crate::models::generation_models::{Product};
use actix_web::{Result};
use sqlx::{Pool, Postgres};
use regex;
use crate::models::generation_models::ParsedResponse;

pub async fn parse_response(response_with_product_suggestions: String, 
  pool: actix_web::web::Data<Pool<Postgres>>) -> Result<ParsedResponse, actix_web::Error>{

// Parses text to extract text and products wrapped in square brackets
let re = regex::Regex::new(r"\[(.*?)\]").unwrap();
  let products: Vec<String> = re.captures_iter(&response_with_product_suggestions)
      .map(|cap| cap[1].trim().to_string())
      .collect();

  // This will change the product's "" to a '' for psql since "" will search columns rather than rows
  let formatted_products_array = format!(
      "{}", 
      products.iter()
          .map(|s| format!("'{}'", s.replace("'", "''"))) // Escape single quotes
          .collect::<Vec<_>>()
          .join(", ")
  );

  // Remove product references from original response
  let cleaned_response = re.replace_all(&response_with_product_suggestions, "")
      .trim()
      .to_string();

  // println!("Extracted products: {:?}", products);

  let query = format!(
      r#"
          SELECT name, description, price, image, handle, vendor FROM public.products
          WHERE seo_title % ANY(ARRAY[{}])
          ORDER BY (
              SELECT MAX(similarity(seo_title, term))
              FROM UNNEST(ARRAY[{}]) AS term
              WHERE seo_title % term
              OR name % term
          ) DESC
          LIMIT 5;
      "#,
      formatted_products_array,
      formatted_products_array
  );

  // Takes extracted product titles and searches the database for them.
  // If none are found then itll just return an empty vector
  let product_rows: Vec<Product> = if !products.is_empty() {
      sqlx::query_as::<_, Product>(&query)
      .bind(&products)
      .fetch_all(pool.get_ref())
      .await
      .map_err(|e| {
          eprintln!("Failed to fetch products: {}", e);
          return actix_web::error::ErrorInternalServerError("Database error")
      })?
  } else {
      Vec::new()
  };

  let mut unique_products:Vec<Product> = Vec::new();
  let mut seen_names: HashSet<String> = std::collections::HashSet::new();

  // Will remove any duplicate products that have been found while chunking
  for product in product_rows {
      if seen_names.insert(product.name.clone()) {
          unique_products.push(product);
      }
  }

    // let store_data = sqlx::query_as::<_, Store>(
    //     "SELECT domain FROM stores WHERE database_table = $1"
    // )
    //     .bind(&selector)
    //     .fetch_one(&pool)
    //     .await
    //     .expect("Couldnt fetch store url");


  // Converts products vector to a valid json array for frontend 
  let json_response = serde_json::to_value(&unique_products).map_err(|_| {
      return actix_web::error::ErrorInternalServerError("Failed to serialize products")
  })?;

  Ok(ParsedResponse {
      text: cleaned_response,
      products: json_response,
      store_domain: "test".to_string()
  })

}