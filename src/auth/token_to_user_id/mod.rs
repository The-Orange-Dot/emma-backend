use uuid::Uuid;
use crate::helpers::modify_types::string_to_uuid;
use actix_web::HttpRequest;
use crate::auth::validate_access_token; 
use jsonwebtoken::errors::ErrorKind;
use actix_web::error::{ErrorUnauthorized, ErrorBadRequest};

pub fn token_to_uuid(req: HttpRequest) -> Result<Uuid, actix_web::Error> {
    let user_id_str: String = token_to_string_id(req)?;

    string_to_uuid(user_id_str)
        .map_err(|e| {
            eprintln!("Failed to parse UUID from token claims: {:?}", e);
            ErrorBadRequest("Invalid user ID format in token claims")
        })
}

pub fn token_to_string_id(req: HttpRequest) -> Result<String, actix_web::Error> {
   
    let access_token_cookie = req.cookie("access_token")
        .ok_or_else(|| {
            eprintln!("Access token cookie not found");
            ErrorUnauthorized("Access token missing. Please log in.")
        })?;


    let token = access_token_cookie.value();

    let claims = match validate_access_token(token) {
        Ok(claims) => claims,
        Err(e) => {
            eprintln!("JWT validation error from access_token cookie: {:?}", e);
            let error_message = match e.kind() {
                ErrorKind::ExpiredSignature => {
                    "Session expired. Please log in again." // More user-friendly
                },
                ErrorKind::InvalidSignature => "Invalid session. Please log in again.",
                ErrorKind::InvalidToken => "Invalid session format. Please log in again.",
                _ => "Session validation failed. Please log in again.",
            };
            return Err(ErrorUnauthorized(error_message));
        }
    };

    Ok(claims.sub)
}
