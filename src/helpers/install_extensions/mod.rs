use sqlx::{postgres::PgPoolOptions};

pub async fn install_extensions(admin_conn_str: &str, db_name: &str) -> Result<(), sqlx::Error> {
    // Create a short-lived pool for just this DB
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{}/{}", admin_conn_str, db_name))  // Connects to `db_name`
        .await?;

    let extensions = ["vector", "vectorscale", "plpython3u", "ai", "pgcrypto", "plpgsql"];

    for extension in extensions {
        let vector_ext = sqlx::query(&format!("CREATE EXTENSION IF NOT EXISTS {}", extension) )
            .execute(&pool)
            .await;

        match vector_ext {
            Ok(_) => {
              println!("Extension '{}' has been installed on {}", extension, &db_name);
            }

            Err(err) => {
              println!("Failed to install extension '{}' on {}: {}", extension, &db_name, err);
            }
        }
    }

    // Pool and connections automatically close when `pool` drops
    Ok(())
}