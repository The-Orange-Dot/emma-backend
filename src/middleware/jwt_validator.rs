use actix_web::{dev::ServiceRequest, HttpMessage, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::auth;

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    match auth::validate_jwt(token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => Err((actix_web::error::ErrorUnauthorized("Invalid token"), req)),
    }
}