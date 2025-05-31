use sqlx::Postgres;
use tokio::task;
use backoff::{ExponentialBackoff, future::retry};
use tokio::time::{Duration, Instant};
use std::panic;
use crate::init::update_embed_data::update_embed_data;

pub async fn attach_embed_data_checker(pool: sqlx::Pool<Postgres>, timer: u64, store_name: String) -> tokio::task::JoinHandle<()> {
    task::spawn(async move {
        panic::set_hook(Box::new(|panic_info| {
            eprintln!("Task panicked: {:?}", panic_info);
        }));

        let operation = || async {
            let result = update_embed_data(pool.clone(), store_name.clone())
                .await;


            match result {
                Ok(res) => {
                    println!("Embedder now checking products on {} ", store_name);
                    Ok(res)
                },
                Err(e) => {
                    // println!("[DEBUG] Update error: {:?}", e);
                    println!("Cannot find table {}", store_name);
                    Err(backoff::Error::transient(e))
                }
            }
        };
        
        let backoff = ExponentialBackoff {
            initial_interval: Duration::from_secs(2),
            multiplier: 2.0,                          
            max_interval: Duration::from_secs(15),    
            max_elapsed_time: Some(Duration::from_secs(7)),
            ..ExponentialBackoff::default()
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
                    println!("Disconnecting embedder for {}", store_name);
                    break;
                }
            }
        }
    })
}