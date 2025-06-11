use sqlx::{Postgres, postgres::PgQueryResult};
use chrono::Utc;

pub async fn update_embed_data(pool: sqlx::Pool<Postgres>, store_name: String) -> Result<PgQueryResult, sqlx::Error> {
    // First, ensure the embeddings table exists
    create_embeddings_table(&pool, &store_name).await?;

    
    let query = format!(
        "
        WITH target_products AS (
            SELECT id, name, price, description, tags, seo_description, seo_title, category, published, status
            FROM {}_products
            WHERE 
                NOT EXISTS (
                    SELECT 1 FROM {}_embeddings 
                    WHERE {}_embeddings.product_id = {}_products.id
                )
                OR updated_at > COALESCE(
                    (SELECT MAX(created_at) FROM {}_embeddings WHERE product_id = {}_products.id),
                    TIMESTAMP '1970-01-01'
                )
            ORDER BY
                CASE WHEN NOT EXISTS (
                    SELECT 1 FROM {}_embeddings 
                    WHERE {}_embeddings.product_id = {}_products.id
                ) THEN 0 ELSE 1 END,
                updated_at ASC
            LIMIT 100
            FOR UPDATE SKIP LOCKED
        ),
        product_chunks AS (
            SELECT 
                id as product_id,
                substring(
                    format('NAME: %s - PRICE: %s - DESCRIPTION: %s %s %s %s - CATEGORY: %s STATUS: %s %s - ',
                    name, price, description, tags, seo_description, seo_title, category, published, status)
                    from (1 + (n-1)*512) for 512
                ) as chunk_text,
                n as chunk_index
            FROM target_products,
            LATERAL (
                SELECT generate_series(
                    1, 
                    ceil(
                        length(
                            format('NAME: %s - PRICE: %s - DESCRIPTION: %s %s %s %s - CATEGORY: %s STATUS: %s %s - ',
                            name, price, description, tags, seo_description, seo_title, category, published, status)
                        )::numeric / 512.0
                    )::integer
                ) as n
            ) chunks
        )
        INSERT INTO {}_embeddings (product_id, chunk_index, chunk_text, embedding, created_at)
        SELECT 
            product_id,
            chunk_index,
            chunk_text,
            ai.ollama_embed('nomic-embed-text', chunk_text),
            $1
        FROM product_chunks
        ON CONFLICT (product_id, chunk_index) 
        DO UPDATE SET
            embedding = EXCLUDED.embedding,
            created_at = EXCLUDED.created_at
        ",
        store_name, 
        store_name,
        store_name, store_name,
        store_name, store_name,
        store_name,
        store_name, store_name,
        store_name
    );

    let current_time = Utc::now();
    let result: Result<PgQueryResult, sqlx::Error> = match sqlx::query(&query)
        .bind(current_time)
        .execute(&pool)
        .await
        {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("Error embedding data on embedding table: {}", err);
                Err(err)
            }
        };

    result
}

async fn create_embeddings_table(pool: &sqlx::Pool<Postgres>, store_name: &str) -> Result<(), sqlx::Error> {

    let query = format!(
        "CREATE TABLE IF NOT EXISTS {}_embeddings (
            id BIGSERIAL PRIMARY KEY,
            product_id NUMERIC NOT NULL REFERENCES {}_products(id),
            chunk_index NUMERIC NOT NULL,
            chunk_text TEXT NOT NULL,
            embedding VECTOR(768) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            UNIQUE(product_id, chunk_index)
        )",
        store_name, store_name
    );
    
    let _ = sqlx::query(&query).execute(pool).await;
    Ok(())

}