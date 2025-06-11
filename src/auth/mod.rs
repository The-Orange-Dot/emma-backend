use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::ErrorKind};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
pub mod token_to_user_id;
pub mod password_encoder;
use actix_web::error::ErrorUnauthorized;
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiration
    pub iat: usize,  // Issued at
    pub typ: String, // "access" or "refresh" tokens
}

// === MODIFIED: Now takes a secret directly ===
pub fn validate_jwt_with_secret(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ).map(|data| data.claims)
}

// This function will now be specifically for access tokens,
// and it should use JWT_ACCESS_SECRET
pub fn validate_access_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    dotenv::dotenv().ok();
    let jwt_access_secret = std::env::var("JWT_ACCESS_SECRET")
        .expect("JWT_ACCESS_SECRET env variable hasnt been set");
    validate_jwt_with_secret(token, &jwt_access_secret)
}


pub fn generate_access_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::minutes(60);

    dotenv::dotenv().ok();
    let jwt_access_secret = std::env::var("JWT_ACCESS_SECRET")
        .expect("JWT_ACCESS_SECRET env variable hasnt been set");

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        typ: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_access_secret.as_ref()),
    )
}

pub fn generate_refresh_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::days(30);

    dotenv::dotenv().ok();
    let jwt_refresh_secret = std::env::var("JWT_REFRESH_SECRET") // === NEW SECRET ===
        .expect("JWT_REFRESH_SECRET env variable hasnt been set");

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        typ: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_refresh_secret.as_ref()),
    )
}

pub fn generate_new_tokens(user_id: &Uuid) -> Result<(String, String), jsonwebtoken::errors::Error> {
    let access_token = generate_access_token(user_id)?;
    let refresh_token = generate_refresh_token(user_id)?;
    Ok((access_token, refresh_token))
}

pub fn verify_refresh_token(token: &str) -> Result<Uuid, actix_web::error::Error> {
    dotenv::dotenv().ok();
    let jwt_refresh_secret = std::env::var("JWT_REFRESH_SECRET") // === NEW SECRET ===
        .expect("JWT_REFRESH_SECRET env variable hasnt been set");

    // Use the specific validate_jwt_with_secret for refresh tokens
    let token_data = validate_jwt_with_secret(token, &jwt_refresh_secret)
        .map_err(|e| {
            eprintln!("Refresh token validation error: {:?}", e);
            let error_message = match e.kind() {
                ErrorKind::ExpiredSignature => "Refresh token expired. Please log in again.",
                ErrorKind::InvalidSignature => "Invalid refresh token signature. Please log in again.",
                ErrorKind::InvalidToken => "Malformed refresh token. Please log in again.",
                _ => "Refresh token validation failed.",
            };
            ErrorUnauthorized(error_message)
        })?;

    if token_data.typ != "refresh" {
       return Err(ErrorUnauthorized("Invalid token type for refresh operation"));
    }

    Uuid::parse_str(&token_data.sub)
        .map_err(|e| {
            eprintln!("Failed to parse UUID from refresh token claims: {:?}", e);
            ErrorUnauthorized("Invalid user ID in refresh token")
        })
}