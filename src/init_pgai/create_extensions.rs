use sqlx::Postgres;

pub async fn create_extensions(pool: sqlx::Pool<Postgres>, extensions : &[&'static str]) {
  
  // Creates extensions if they dont exist
  // Required functions should be 'ai,' 'vector,' and 'vectorscale'
  for extension in extensions {
    println!("[ Creating Extension '{}' in database ]", extension);

    let query = "CREATE EXTENSION IF NOT EXISTS * CASCADE;";
    let updated_query = query.replace("*", extension);

    let _create_db = sqlx::query(&updated_query)
        .bind(extension.to_string())
        .execute(&pool)
        .await
        .map_err(|err| {
          eprint!("[ERROR] Error creating extension {}: {}. ", extension, err);
          actix_web::error::ErrorInternalServerError("[ERROR] Error creating extension in table")
        });

    println!("");

  }
}