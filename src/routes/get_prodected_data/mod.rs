use crate::auth::token_extractor::AccessTokenFromCookie; 

#[get("/protected_data")] 
async fn get_protected_data(token_from_cookie: AccessTokenFromCookie) -> impl Responder {
    println!("Access token received from cookie: {}", token_from_cookie.0);
    HttpResponse::Ok().json(serde_json::json!({"message": "This is protected data!", "token_received": token_from_cookie.0}))
}