use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiration
    pub iat: usize,  // Issued at
}

pub fn create_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(24); // 24-hour expiration
    
    dotenv::dotenv().ok();
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET env variable hasnt been set");

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    dotenv::dotenv().ok();
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET env variable hasnt been set");    
        
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    ).map(|data| data.claims)
}

