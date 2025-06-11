use actix_web::{
    HttpResponse,
    post,
    cookie::{Cookie, time::Duration, SameSite}
};

#[post("/logout")]
pub async fn logout_account() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("access_token", "") 
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .path("/")
                // .domain("100.125.54.38") 
                .max_age(Duration::new(0, 0)) 
                .finish()
        )
        .cookie(
            Cookie::build("refresh_token", "") 
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .path("/refresh") 
                // .domain("100.125.54.38") 
                .max_age(Duration::new(0, 0))
                .finish()
        )
        .json(serde_json::json!({
            "status": 200,
            "message": "Successfully logged out!"
        })))
}