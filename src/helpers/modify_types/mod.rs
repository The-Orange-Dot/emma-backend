use uuid::Uuid;

pub fn string_to_uuid(id_string: String) -> Uuid {
  Uuid::parse_str(&id_string)
      .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))
      .expect("Failed to parse UUID")
}