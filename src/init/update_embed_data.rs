use sqlx::Postgres;
use tokio::task;
use backoff::{ExponentialBackoff, future::retry};
use tokio::time::{Duration, Instant};
use std::panic;
use chrono::Utc;

// NEEDED TO DO:
// IF THE PRODUCT HAS NO EMBEDDED VALUE IN EMBEDDING COL - ADD IT
// IF THE PRODUCT HAS BEEN UPDATED WITHIN THE TIME BETWEEN THE RUNS - UPDATE IT

pub async fn update_embed_data(pool: sqlx::Pool<Postgres>, timer: u64, store_name: String) -> tokio::task::JoinHandle<()> {
    task::spawn(async move {
        // Add panic hook to catch any silent failures
        panic::set_hook(Box::new(|panic_info| {
            eprintln!("Task panicked: {:?}", panic_info);
        }));

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

        let operation = || async {
            let current_time = Utc::now().naive_utc();
            let result = sqlx::query(&query)
            .bind(current_time)
            .execute(&pool)
            .await;

            match result {
                Ok(res) => {
                    // println!("Embedder now checking products on {} ", store_name);
                    Ok(res)
                },
                Err(e) => {
                    println!("[DEBUG] Update error: {:?}", e);
                    Err(backoff::Error::transient(e))
                }
            }
        };
        
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(timer)),
            ..Default::default()
        };

        loop {
            let start = Instant::now();

            match retry(backoff.clone(), operation).await {
                Ok(result) => {
                    let rows = result.rows_affected();
                    if rows != 0 {
                      println!("[SUCCESS] Processed {} rows in {:?}", rows, start.elapsed());
                    }
                    
                    if rows == 0 {
                        // println!("[SLEEP] No rows to process, sleeping...");
                        tokio::time::sleep(Duration::from_secs(timer)).await;
                    }
                }
                Err(e) => {
                    println!("[FATAL ERROR] {:?}", e);
                    tokio::time::sleep(Duration::from_secs(timer)).await;
                }
            }
        }
    })
}