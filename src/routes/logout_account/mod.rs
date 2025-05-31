use actix_web::{post, HttpResponse, Responder, cookie::{Cookie, time::Duration},};

#[post("/logout")]
pub async fn logout_account() -> impl Responder {
    HttpResponse::Ok()
        .cookie(
            Cookie::build("jwt", "") 
                .path("/")
                .max_age(Duration::seconds(0)) 
                .finish()
        )
        .json(serde_json::json!({
            "status": "success", 
            "message": "User has logged out",
            "response": []
        }))
}