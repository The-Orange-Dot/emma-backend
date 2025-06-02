use sqlx::{Postgres, postgres::PgQueryResult};
use chrono::Utc;

pub async fn update_embed_data(pool: sqlx::Pool<Postgres>, store_name: String) -> Result<PgQueryResult, sqlx::Error> {
        
        let query = format!(
            "
                WITH target_products AS (
                    SELECT id 
                    FROM {}_products
                    WHERE 
                        EMBEDDING IS NULL 
                        OR updated_at > COALESCE(
                            (SELECT MAX(updated_at) FROM {}_products),
                            TIMESTAMP '1970-01-01'
                        )
                    ORDER BY
                        CASE WHEN EMBEDDING IS NULL THEN 0 ELSE 1 END,
                        updated_at ASC
                    LIMIT 100
                    FOR UPDATE SKIP LOCKED
                )
                UPDATE {}_products 
                SET 
                    EMBEDDING = ai.ollama_embed(
                        'nomic-embed-text', 
                        format('NAME: %s - 
                        PRICE: %s - 
                        DESCRIPTION: %s %s %s %s - 
                        CATEGORY: %s 
                        STATUS: %s %s - ', 
                        name, price, description, tags, seo_description, 
                        seo_title, category, published, status)
                    ),
                    updated_at = $1
                WHERE id IN (SELECT id FROM target_products)
            ", 
            store_name, 
            store_name, 
            store_name
        );

            let current_time = Utc::now();
            let result = sqlx::query(&query)
            .bind(current_time)
            .execute(&pool)
            .await;

            result
}