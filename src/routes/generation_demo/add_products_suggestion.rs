use actix_web::{Result, Error};
use sqlx::{Pool, Postgres};
use crate::models::{generation_models::DemoPayload, store_models::Store};
use tokio::time::timeout;
use std::time::Duration;

pub async fn add_products_suggestion(
    req: DemoPayload,
    pool: Pool<Postgres>,
) -> Result<String, Error> {
    dotenv::dotenv().ok();
    let model = std::env::var("LLM_MODEL")
        .expect("No model has been set for PGAI context prompt");
    // Prepare base64 images for PostgreSQL
    let image_params = if !req.images.is_empty() {
        let image_strings = req.images.iter()
            .map(|img| {
                let clean_base64 = if img.data_url.starts_with("data:") {
                    img.data_url.split(',').nth(1).unwrap_or(&img.data_url)
                } else {
                    &img.data_url
                };
                format!("decode('{}', 'base64')", clean_base64)
            })
            .collect::<Vec<_>>()
            .join(",");
        
        format!("ARRAY[{}]", image_strings)
    } else {
        "NULL::bytea[]".to_string()
    };  

    let store = sqlx::query_as::<_, Store>("SELECT * FROM stores WHERE store_table = $1")
        .bind(&req.selector)
        .fetch_one(&pool)
        .await
        .map_err(|err| {
            eprint!("Error fetching store data: {}", err);
        }).unwrap();
        
    let formatted_sys_prompt = store.sys_prompt.replace("'", "''");

    let query = format!(
        r#"
        WITH product_embeddings AS (
            SELECT 
                p.id,
                p.name,
                p.seo_title,
                p.description,
                p.category,
                p.tags,
                p.seo_description,
                p.vendor,
                e.embedding
            FROM {}_products p
            JOIN {}_embeddings e ON p.id = e.product_id
        ),
        relevant_chunks AS (
            SELECT 
                pe.id,
                pe.name,
                pe.seo_title,
                pe.tags,
                pe.category,
                pe.seo_description,
                pe.vendor,
                pe.description,
                pe.embedding <=> ai.ollama_embed('nomic-embed-text', $1) as distance
            FROM product_embeddings pe
            ORDER BY distance
            LIMIT 50
        ),
        relevant_products AS (
            SELECT 
                id,
                name,
                description,
                tags,
                category,
                seo_description,
                seo_title,
                vendor,
                MIN(distance) as min_distance
            FROM relevant_chunks
            GROUP BY id, name, description, tags, seo_description, seo_title, vendor, category
            ORDER BY min_distance
            LIMIT 15
        ),
        context_agg AS (
            SELECT string_agg(name || ' - ' || vendor || '-' || seo_title, ', ') AS context_chunk
            FROM relevant_products
        )
        SELECT ai.ollama_generate(
            '{}',
            'You should always add products from the context in your response ' ||
            'Suggest up to 3-6 products from the context in the your response ' ||
            'You dont need to compliment the client.' ||
            'Any products you suggest in context should be in bold text' ||
            'You must always include the context products you suggested as a list at the end of the response each have to be in their own square brackets with no commas between them [Like] [This].\n\n' ||
            'User query: ' || $1 || '\n\n' ||
            'Relevant products: ' || (SELECT context_chunk FROM context_agg),
            system_prompt => 'Your job is to look at the image given to you and answer any questions that are asked. 
            You need to talk to the client as you are a specialist or guru consulting them.
            Never acknowledge that you are a chatbot or ai. 
            Never include links in your response.
            You must never say that you are suggesting products. 
            Just include it in the conversation organically.
            You must keep your answers at under 200 words but over 150 words. {}',
            images => {}
        )->>'response' as response
        "#,
        req.selector,
        req.selector,
        model,
        formatted_sys_prompt,
        image_params
    );

    let query_future = sqlx::query_scalar::<_, String>(&query)
        .bind(req.prompt)
        .fetch_one(&pool);


    let response = timeout(Duration::from_secs(30), query_future)
        .await
        .map_err(|_| {
            eprintln!("Database query timed out after 10 seconds.");
            actix_web::error::ErrorRequestTimeout("Database query timed out")
        })?
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            actix_web::error::ErrorInternalServerError("Failed to generate product suggestions")
        })?;

    Ok(response)
}