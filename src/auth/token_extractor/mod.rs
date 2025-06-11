use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    FromRequest, HttpRequest,
};
use futures::future::{Ready, ok, err};

#[derive(Debug, Clone)] 
pub struct AccessTokenFromCookie(pub String);

impl FromRequest for AccessTokenFromCookie {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(cookie) = req.cookie("access_token") {
            let token_value = cookie.value().to_string();

            if !token_value.is_empty() {

                return ok(AccessTokenFromCookie(token_value)); // If valid
            }
        }
        err(ErrorUnauthorized("Access token cookie not found or invalid"))
    }
}