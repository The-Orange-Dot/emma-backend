use uuid::Uuid;
use crate::helpers::modify_types::string_to_uuid;
use actix_web::{HttpRequest, HttpResponse, Result, cookie::{Cookie, time::Duration},};
use crate::auth::validate_jwt;

fn logout() -> HttpResponse {
      HttpResponse::Unauthorized()
        .cookie(
            Cookie::build("jwt", "") 
                .path("/")
                .max_age(Duration::seconds(0)) 
                .finish()
        )
        .json(serde_json::json!({
            "status": 401, 
            "message": "Missing token, flushing cookies",
            "response": []
        }))
}

pub fn token_to_uuid(req: HttpRequest) -> Result<Uuid, HttpResponse> {

    

   let auth_header = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(logout()),
    };

    let claims = match validate_jwt(token) {
        Ok(claims) => claims,
        Err(_) => return Err(logout()),
    };

    let user_id = claims.sub;

    Ok(string_to_uuid(user_id.to_string()))
}

pub fn token_to_string_id(req: HttpRequest) -> Result<String, HttpResponse> {

   let auth_header = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok());

    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(HttpResponse::Unauthorized().json("Missing or invalid Authorization header")),
    };

    let claims = match validate_jwt(token) {
        Ok(claims) => claims,
        Err(_) => return Err(HttpResponse::Unauthorized().json("Invalid token")),
    };

       Ok(claims.sub)

}