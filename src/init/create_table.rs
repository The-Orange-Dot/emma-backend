pub async fn create_table (pool: sqlx::Pool<sqlx::Postgres>, table_name: &str, columns: &[&'static str] ) {

  // Create a table  768 STRUCTURE
  let mut query = format!("CREATE TABLE IF NOT EXISTS {} (", table_name);

  for column in columns {
    query.push_str(column);
    query.push_str(", ");
  }

  query.push_str("embedding vector(768))");
  
  let _create_ai_table = sqlx::query("CREATE TABLE IF NOT EXISTS products (id SERIAL PRIMARY KEY, name TEXT, description TEXT, embedding vector(768))")
    .execute(&pool)
    .await
    .map_err(|err| {
      eprint!("ERROR: Error creating table 'products': {}", err);
      actix_web::error::ErrorInternalServerError(" 1. ERROR: Error creating table 'products'")
    });
}