use actix_web::{Result, Error};
use sqlx::{Pool, Postgres};
use crate::models::generation_models::{Payload};

pub async fn add_products_suggestion(
    req: Payload,
    pool: actix_web::web::Data<Pool<Postgres>>,
    _table: &str,
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

    let default_system_val = format!("");
    let sys_prompt = req.system_prompt.unwrap_or(default_system_val); 

    let query = format!(
        r#"
        WITH relevant_products AS (
            SELECT *
            FROM public.products
            ORDER BY embedding <=> ai.ollama_embed('nomic-embed-text', $1)
            LIMIT 10
        ),
        context_agg AS (
            SELECT string_agg(name || ' - ' || seo_title, ', ') AS context_chunk
            FROM relevant_products
        )
        SELECT ai.ollama_generate(
            '{}',
            'You should add products from the context in your response ' ||
            'Try to suggest up to 3 products, but no more than 4. ' ||
            'You dont need to compliment the client.' ||
            'Include the products as a list at the end of the response each have to be in their own square brackets with no commas between them [Like] [This].\n\n' ||
            'User query: ' || $1 || '\n\n' ||
            'Relevant products: ' || (SELECT context_chunk FROM context_agg),
            system_prompt => 'Your job is to look at the image given to you and answer any questions that are asked. You must NEVER give medical advice to the clients. {}',
            images => {}
        )->>'response' as response
        "#,
        model,
        sys_prompt,
        image_params
    );

    // println!("Executing query:\n{}", query); // Debug output

    let response = sqlx::query_scalar::<_, String>(&query)
        .bind(req.prompt)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}\nQuery: {}", err, query);
            actix_web::error::ErrorInternalServerError("Failed to generate product suggestions")
        })?;

    Ok(response)
}