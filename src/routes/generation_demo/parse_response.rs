use std::collections::HashSet;
use crate::models::{generation_models::Product, store_models::Store};
use actix_web::{Result, Error};
use sqlx::{Pool, Postgres};
use regex;
use crate::models::generation_models::ParsedResponse;

pub async fn parse_response(
    response_with_product_suggestions: String, 
    pool: Pool<Postgres>,
    selector: String
) -> Result<ParsedResponse, Error>{

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

    println!("Extracted products: {:?}", formatted_products_array);

    // Remove product references from original response
    let cleaned_response = re.replace_all(&response_with_product_suggestions, "")
        .trim()
        .to_string();

    let query = format!(
        r#"
        SELECT name, description, price, product_url, image, handle, vendor
        FROM {}_products
        WHERE seo_title % ANY($1::text[]) OR name % ANY($1::text[])
        ORDER BY GREATEST(
            (SELECT MAX(similarity(seo_title, term)) FROM UNNEST($1::text[]) AS term WHERE seo_title % term),
            (SELECT MAX(similarity(name, term)) FROM UNNEST($1::text[]) AS term WHERE name % term)
        ) DESC
        LIMIT 6;
        "#,
        selector
    );
        
    // Takes extracted product titles and searches the database for them.
    // If none are found then itll just return an empty vector
    let product_rows: Vec<Product> = if !products.is_empty() {
        sqlx::query_as::<_, Product>(&query)
        .bind(&products)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch products: {}", e);
            return actix_web::error::ErrorInternalServerError("Database error")
        })?
    } else {
        Vec::new()
    };

    // println!("DEBUG: {:?}", product_rows[0] );
    
    // Fetches store data to send domain to the front
    let store_data = match sqlx::query_as::<_, Store>(
        "SELECT * FROM stores WHERE store_table = $1"
    )
        .bind(&selector)
        .fetch_one(&pool)
        .await
        {
            Ok(res) => res,
            Err(sqlx::Error::RowNotFound) => {
                eprint!("Store not found");
                
                return Err(actix_web::error::ErrorNotFound(format!("No stores have been found")));
            }
            Err(err) => {
                eprint!("Internal Error finding table");
                return Err(actix_web::error::ErrorInternalServerError(format!("Failed to find store: {}", err)));
            }    
        };

    // println!("DEBUG: {:?}", store_data);

    let mut unique_products:Vec<Product> = Vec::new();
    let mut seen_names: HashSet<String> = std::collections::HashSet::new();

    // Will remove any duplicate products that have been found while chunking
    for product in product_rows {
        if seen_names.insert(product.name.clone()) {
            unique_products.push(product);
        }
    }

    // Converts products vector to a valid json array for frontend 
    let json_response = match serde_json::to_value(&unique_products)
    {
        Ok(res) => res,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!("Failed to serialize products: {}", err)))
        }
    };

    Ok(ParsedResponse {
        text: cleaned_response,
        products: json_response,
        store_domain: store_data.domain,
    })

}