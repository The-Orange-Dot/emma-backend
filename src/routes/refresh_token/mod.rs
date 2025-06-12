use actix_web::{post, HttpRequest, HttpResponse, cookie::{Cookie, time::Duration, SameSite}};
use crate::auth::{verify_refresh_token, generate_new_tokens};
use serde_json::json;

#[post("/refresh")]
pub async fn refresh_token(req: HttpRequest) -> HttpResponse {
    let refresh_token_cookie = match req.cookie("refresh_token") {
        Some(cookie) => cookie,
        None => {
            eprintln!("Refresh token cookie not found during refresh attempt.");
            return HttpResponse::Unauthorized().json(json!({
                "status": 401,
                "message": "Refresh token not found",
            }));
        }
    };

    let old_refresh_token_value = refresh_token_cookie.value();

    let user_uuid = match verify_refresh_token(old_refresh_token_value) {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!("Invalid refresh token during refresh: {:?}", e);
            return HttpResponse::Unauthorized()
                .cookie(Cookie::build("access_token", "")
                    .max_age(Duration::new(0,0))
                    .path("/refresh")
                    .secure(true)
                    .domain("meetemma.ai")                 
                    .http_only(true)
                    .same_site(SameSite::None)
                    .finish())
                .cookie(Cookie::build("refresh_token", "")
                    .max_age(Duration::new(0,0))
                    .path("/refresh")
                    .secure(true)
                    .domain("meetemma.ai") 
                    .http_only(true)
                    .same_site(SameSite::None)
                    .finish())
                .json(json!({
                    "status": 401,
                    "message": "Invalid or expired refresh token. Please log in again.",
                }));
        }
    };

    let (new_access_token, new_refresh_token) = match generate_new_tokens(&user_uuid) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to generate new tokens after refresh: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": 500,
                "message": "Failed to generate new tokens",
            }));
        }
    };

    HttpResponse::Ok()
        .cookie(
            Cookie::build("access_token", &new_access_token)
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .path("/refresh")
                .domain("localhost")
                .max_age(Duration::minutes(60))
                .finish()
        )
        .cookie(
            Cookie::build("refresh_token", &new_refresh_token)
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .path("/refresh")
                .domain("localhost")
                .max_age(Duration::days(30))
                .finish()
        )
        .json(json!({
            "status": 200,
            "message": "Tokens refreshed successfully!",
        }))
}