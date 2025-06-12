use actix_web::{
    HttpResponse,
    post,
    cookie::{Cookie, time::Duration, SameSite}
};

#[post("/logout")]
pub async fn logout_account() -> Result<HttpResponse, actix_web::Error> {
    dotenv::dotenv().ok();
    let is_development = std::env::var("APP_ENV")
        .map(|s| s == "development")
        .unwrap_or(false);

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("access_token", "") 
                .http_only(true)
                .secure(if is_development { false } else { true })
                .same_site(if is_development { SameSite::Lax } else { SameSite::None })
                .path("/")
                .domain(if is_development {"localhost"} else {"meetemma.ai"}) 
                .max_age(Duration::new(0, 0)) 
                .finish()
        )
        .cookie(
            Cookie::build("refresh_token", "") 
                .http_only(true)
                .secure(if is_development { false } else { true })
                .same_site(if is_development { SameSite::Lax } else { SameSite::None })
                .path("/refresh") 
                .domain(if is_development {"localhost"} else {"meetemma.ai"}) 
                .max_age(Duration::new(0, 0))
                .finish()
        )
        .json(serde_json::json!({
            "status": 200,
            "message": "Successfully logged out!"
        })))
}